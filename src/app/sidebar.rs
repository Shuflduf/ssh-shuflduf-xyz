use async_trait::async_trait;
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, make_block},
    types::{Focus, ServerState, Sidebar},
};

#[async_trait]
impl TerminalPane for Sidebar {
    async fn handle_key(
        &mut self,
        key_code: KeyCode,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_item += 2;
                self.selected_item %= 3;
                *needs_redraw = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_item += 1;
                self.selected_item %= 3;
                *needs_redraw = true;
            }
            _ => {}
        }
    }
}

impl StatefulWidget for &Sidebar {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let items = ["One", "Two", "Three"];
        let lines = items
            .iter()
            .enumerate()
            .map(|(i, &text)| {
                let text_col = if i == self.selected_item {
                    Color::White
                } else {
                    Color::DarkGray
                };
                Line::from(text).set_style(Style::new().fg(text_col))
            })
            .collect::<Vec<Line>>();

        Paragraph::new(lines)
            .block(
                make_block(*focus == Focus::Pane, 1, "Navigation").title_bottom(
                    Line::from(vec![" [Q]".blue().bold(), " Quit ".into()]).centered(),
                ),
            )
            .render(area, buf);
    }
}
