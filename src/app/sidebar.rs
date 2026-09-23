use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};
use russh::client::KeyboardInteractiveAuthResponse;
use strum::{EnumCount, IntoEnumIterator, VariantArray};

use crate::{
    app::{TerminalPane, make_block},
    types::{Focus, MainPane, ServerState, Sidebar, SidebarItem},
};

#[async_trait]
impl TerminalPane for Sidebar {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        current_pane: &mut MainPane,
        focus: &mut Focus,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.item = SidebarItem::VARIANTS[(SidebarItem::VARIANTS
                    .iter()
                    .position(|&item| item == self.item)
                    .unwrap()
                    + SidebarItem::COUNT
                    - 1)
                    % SidebarItem::COUNT];
                *needs_redraw = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.item = SidebarItem::VARIANTS[(SidebarItem::VARIANTS
                    .iter()
                    .position(|&item| item == self.item)
                    .unwrap()
                    + 1)
                    % SidebarItem::COUNT];
                *needs_redraw = true;
            }
            KeyCode::Enter => {
                *current_pane = match self.item {
                    SidebarItem::Counter => MainPane::Counter,
                    SidebarItem::Games => MainPane::Games,
                };
                *needs_redraw = true;
                *focus = Focus::Pane;
            }
            _ => {}
        }
    }
}

impl StatefulWidget for &Sidebar {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let lines = SidebarItem::iter()
            .map(|item| {
                let text_col = if item == self.item {
                    Color::White
                } else {
                    Color::DarkGray
                };
                Line::from(item.to_string()).set_style(Style::new().fg(text_col))
            })
            .collect::<Vec<Line>>();

        Paragraph::new(lines)
            .block(
                make_block(*focus == Focus::Sidebar, 1, "Navigation").title_bottom(
                    Line::from(vec![" [Ctrl+Q]".blue().bold(), " Quit ".into()]).centered(),
                ),
            )
            .render(area, buf);
    }
}
