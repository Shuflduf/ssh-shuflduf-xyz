use std::collections::HashMap;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, layout::Rect, widgets::Clear,
};
use russh::{
    Channel, ChannelId, ChannelWriteHalf, Pty,
    keys::PublicKey,
    server::{Auth, ChannelOpenHandle, Config, Handler, Msg, Server, Session},
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

use crate::types::{
    AppServer, AppState, ClientEvent, Command, InputParser, Message, ServerState, SshTerminal,
    TerminalHandle,
};

impl AppServer {
    pub async fn run(&mut self) -> Result<()> {
        let config = Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![
                russh::keys::PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519)
                    .unwrap(),
            ],
            nodelay: true,
            ..Default::default()
        };

        self.run_on_address(Arc::new(config), ("0.0.0.0", 2222))
            .await?;
        Ok(())
    }
}

impl Server for AppServer {
    type Handler = Self;

    fn new_client(&mut self, _peer_address: Option<std::net::SocketAddr>) -> Self {
        let per_connection_handler = self.clone();
        self.client_id += 1;
        per_connection_handler
    }
}

impl Handler for AppServer {
    type Error = color_eyre::eyre::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let (channel_read, channel) = channel.split();
        drop(channel_read);
        let terminal_handle = TerminalHandle::start(session.handle(), channel.id()).await;
        let terminal = Terminal::with_options(
            CrosstermBackend::new(terminal_handle),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::default()),
            },
        )?;

        let (client_event_sender, client_event_receiver) = unbounded_channel();
        tokio::spawn(client_event_loop(
            self.client_id,
            self.clients.clone(),
            self.server_state.clone(),
            channel,
            terminal,
            AppState::default(),
            client_event_receiver,
        ));

        self.clients
            .lock()
            .await
            .insert(self.client_id, client_event_sender);

        reply.accept().await;
        Ok(())
    }

    async fn auth_publickey(
        &mut self,
        _username: &str,
        _public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_none(&mut self, _username: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let _ = session.channel_success(channel);
        Ok(())
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        incoming_bytes: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(client_sender) = self.clients.lock().await.get(&self.client_id) {
            let _ = client_sender.send(ClientEvent::Input(incoming_bytes.to_vec()));
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        column_count: u32,
        row_count: u32,
        _pixel_width: u32,
        _pixel_height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let new_size_rect = Rect {
            x: 0,
            y: 0,
            width: column_count as u16,
            height: row_count as u16,
        };
        self.notify_client_of_resize(new_size_rect).await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        column_count: u32,
        row_count: u32,
        _pixel_width: u32,
        _pixel_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let new_size_rect = Rect {
            x: 0,
            y: 0,
            width: column_count as u16,
            height: row_count as u16,
        };
        self.notify_client_of_resize(new_size_rect).await;

        session.channel_success(channel)?;
        Ok(())
    }
}

impl AppServer {
    async fn notify_client_of_resize(&self, new_size_rect: Rect) {
        if let Some(client_sender) = self.clients.lock().await.get(&self.client_id) {
            let _ = client_sender.send(ClientEvent::Resize(new_size_rect));
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn client_event_loop(
    client_id: usize,
    clients: Arc<Mutex<HashMap<usize, UnboundedSender<ClientEvent>>>>,
    server_state: ServerState,
    client_channel: ChannelWriteHalf<Msg>,
    mut terminal: SshTerminal,
    mut app: AppState,
    mut client_event_receiver: UnboundedReceiver<ClientEvent>,
) {
    let mut input_parser = InputParser::default();
    let mut server_command_receiver = server_state.broadcast_sender.subscribe();
    let (mut needs_redraw, mut clear_screen) = (true, true);

    loop {
        tokio::select! {
            client_event = client_event_receiver.recv() => match client_event {
                Some(client_event) => {
                    for message in handle_event(client_event, &mut input_parser) {
                        handle_message(
                            message,
                            &mut app,
                            &server_state,
                            &mut needs_redraw,
                            &mut terminal,
                            &mut clear_screen,
                        )
                        .await;
                    }
                }
                None => break,
            },
            server_command = server_command_receiver.recv() => match server_command {
                Ok(command) => {
                    let previous_counter = app.counter;
                    app.apply_command(&command);
                    if app.counter != previous_counter {
                        needs_redraw = true;
                    }
                }
                Err(_) => {
                    app.counter = *server_state.current_value.lock().await;
                    needs_redraw = true;
                }
            },
        }

        if app.should_exit {
            let _ = client_channel.close().await;
            break;
        }

        if needs_redraw {
            let clear_this_frame = clear_screen;
            clear_screen = false;
            let _ = terminal.draw(|frame| {
                if clear_this_frame {
                    frame.render_widget(Clear, frame.area());
                }
                app.draw(frame);
            });
            needs_redraw = false;
        }
    }

    clients.lock().await.remove(&client_id);
}

fn handle_event(client_event: ClientEvent, input_parser: &mut InputParser) -> Vec<Message> {
    match client_event {
        ClientEvent::Input(incoming_bytes) => input_parser
            .feed(&incoming_bytes)
            .into_iter()
            .map(|key_event| Message::KeyPressed(key_event.code))
            .collect(),
        ClientEvent::Resize(new_size_rect) => vec![Message::TerminalResized(new_size_rect)],
    }
}

async fn handle_message(
    message: Message,
    app: &mut AppState,
    server_state: &ServerState,
    needs_redraw: &mut bool,
    terminal: &mut SshTerminal,
    clear_screen: &mut bool,
) {
    match message {
        Message::KeyPressed(key_code) => match key_code {
            KeyCode::Char('q') => app.should_exit = true,
            KeyCode::Left | KeyCode::Right => {
                let command = reduce_counter_command(server_state, key_code).await;
                app.apply_command(&command);
                let _ = server_state.broadcast_sender.send(command);
                *needs_redraw = true;
            }
            _ => {}
        },
        Message::TerminalResized(new_size_rect) => {
            let _ = terminal.resize(new_size_rect);
            *needs_redraw = true;
            *clear_screen = true;
        }
    }
}

async fn reduce_counter_command(server_state: &ServerState, key_code: KeyCode) -> Command {
    let mut counter = server_state.current_value.lock().await;
    match key_code {
        KeyCode::Right => *counter += 1,
        KeyCode::Left => {
            if *counter > 0 {
                *counter -= 1;
            }
        }
        _ => unreachable!(),
    }
    Command::CounterChanged { value: *counter }
}

impl Drop for AppServer {
    fn drop(&mut self) {
        let disconnected_client_id = self.client_id;
        let clients = self.clients.clone();
        tokio::spawn(async move {
            clients.lock().await.remove(&disconnected_client_id);
        });
    }
}
