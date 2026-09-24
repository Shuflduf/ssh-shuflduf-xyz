use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::Block,
};

use crate::{
    colours::SCHEME,
    types::{ClientState, Focus, MainPane, ServerMessage, ServerState},
};

mod counter;
mod games;
mod sidebar;

#[async_trait]
pub trait TerminalPane: Send {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        current_pane: &mut MainPane,
        focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    );
}

impl ClientState {
    pub fn apply_command(&mut self, command: &ServerMessage) {
        match command {
            ServerMessage::Increment => self.counter.count += 1,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let layout =
            Layout::horizontal([Constraint::Max(30), Constraint::Fill(1)]).split(frame.area());

        frame.render_stateful_widget(&self.sidebar, layout[0], &mut self.focus);
        match self.current_pane {
            MainPane::Counter => {
                frame.render_stateful_widget(&self.counter, layout[1], &mut self.focus);
            }
            MainPane::Games => {
                frame.render_stateful_widget(&self.games, layout[1], &mut self.focus);
            }
        }
        // frame.render_widget(&self.counter, layout[1]);
    }

    pub fn set_focus(&mut self, focus: Focus, needs_redraw: &mut bool) {
        self.focus = focus;
        *needs_redraw = true;
    }

    pub async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match self.focus {
            Focus::Sidebar => {
                self.sidebar
                    .handle_key(
                        key_event,
                        &mut self.current_pane,
                        &mut self.focus,
                        server_state,
                        needs_redraw,
                    )
                    .await;
            }
            Focus::Pane => match self.current_pane {
                MainPane::Counter => {
                    self.counter
                        .handle_key(
                            key_event,
                            &mut self.current_pane,
                            &mut self.focus,
                            server_state,
                            needs_redraw,
                        )
                        .await;
                }
                MainPane::Games => {
                    self.games
                        .handle_key(
                            key_event,
                            &mut self.current_pane,
                            &mut self.focus,
                            server_state,
                            needs_redraw,
                        )
                        .await;
                }
            },
        }
    }
}

fn border_col(focused: bool) -> Color {
    if focused {
        SCHEME.text
    } else {
        SCHEME.text_secondary
    }
}

pub fn key_label<'a>(key: &'a str, label: &'a str) -> Line<'a> {
    Line::from(vec![
        format!(" [{key}]").fg(SCHEME.keys).bold(),
        format!(" {label} ").bold(),
    ])
}

pub fn make_block(focused: bool) -> Block<'static> {
    Block::bordered()
        .bg(SCHEME.base)
        .border_style(Style::new().fg(border_col(focused)))
        .border_set(border::ROUNDED)
}
