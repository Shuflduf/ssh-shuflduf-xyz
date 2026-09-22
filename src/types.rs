use std::{collections::HashMap, sync::Arc};

use crossterm::event::KeyCode;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use tokio::sync::{Mutex, broadcast, mpsc::UnboundedSender};

pub type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

#[derive(Default)]
pub struct AppState {
    pub counter: i32,
    pub should_exit: bool,
}

pub enum ClientEvent {
    Input(Vec<u8>),
    Resize(Rect),
}

#[derive(Clone)]
pub enum Message {
    KeyPressed(KeyCode),
    TerminalResized(Rect),
}

#[derive(Clone)]
pub enum Command {
    CounterChanged { value: i32 },
}

#[derive(Clone)]
pub struct ServerState {
    pub current_value: Arc<Mutex<i32>>,
    pub broadcast_sender: broadcast::Sender<Command>,
}

impl Default for ServerState {
    fn default() -> Self {
        let (broadcast_sender, _receiver) = broadcast::channel(64);
        Self {
            current_value: Arc::new(Mutex::new(0)),
            broadcast_sender,
        }
    }
}

pub struct TerminalHandle {
    pub terminal_bytes_sender: UnboundedSender<Vec<u8>>,
    pub pending_bytes: Vec<u8>,
}

#[derive(Default)]
pub struct InputParser {
    pub pending_bytes: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct AppServer {
    pub clients: Arc<Mutex<HashMap<usize, UnboundedSender<ClientEvent>>>>,
    pub server_state: ServerState,
    pub client_id: usize,
}
