use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::Block,
};

use crate::types::{ClientState, ServerMessage, ServerState};

mod counter;
mod games;
mod sidebar;

#[async_trait]
pub trait TerminalPane: Send {
    async fn handle_key(
        &mut self,
        key_code: KeyCode,
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

    pub fn draw(&self, frame: &mut Frame) {
        let layout =
            Layout::horizontal([Constraint::Max(30), Constraint::Fill(1)]).split(frame.area());

        frame.render_widget(&self.sidebar, layout[0]);
        frame.render_widget(&self.games, layout[1]);
        // frame.render_widget(&self.counter, layout[1]);
    }

    pub fn set_focus(&mut self, index: u8, needs_redraw: &mut bool) {
        self.sidebar.focused = false;
        self.counter.focused = false;
        match index {
            1 => self.sidebar.focused = true,
            2 => self.counter.focused = true,
            _ => unreachable!(),
        }
        *needs_redraw = true;
    }

    pub async fn handle_key(
        &mut self,
        key_code: KeyCode,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        if self.sidebar.focused {
            self.sidebar
                .handle_key(key_code, server_state, needs_redraw)
                .await;
        }
        if self.counter.focused {
            self.counter
                .handle_key(key_code, server_state, needs_redraw)
                .await;
        }
    }
}

fn make_block(focused: bool, index: u8, title: &str) -> Block<'_> {
    let border_col = if focused {
        Color::White
    } else {
        Color::DarkGray
    };
    Block::bordered()
        .title(
            Line::from(vec![
                format!(" [{index}]").blue().bold(),
                format!(" {title} ").into(),
            ])
            .centered(),
        )
        .border_style(Style::new().fg(border_col))
        .border_set(border::ROUNDED)
}
