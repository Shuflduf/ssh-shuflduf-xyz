use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Content, Focus, ServerState},
};

#[derive(Default)]
pub struct Chat {
    current_input: String,
    cursor_pos: usize,
}

#[async_trait]
impl TerminalPane for Chat {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        current_pane: &mut Content,
        focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match key_event.code {
            KeyCode::Char(c) => {
                // self.current_input += &c.to_string();
                self.current_input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                *needs_redraw = true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.current_input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    *needs_redraw = true
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    *needs_redraw = true
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.current_input.len() {
                    self.cursor_pos += 1;
                    *needs_redraw = true
                }
            }
            _ => {}
        }
    }
}

impl StatefulWidget for &Chat {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Chat").centered())
            .render(area, buf);

        let line_length = (area.width - 8) as f32;
        let text_height = (((self.current_input.len() as f32) / line_length).ceil() as u16).max(1);

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(text_height + 2)])
            .spacing(1)
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(area);

        let input_text = Layout::vertical([Constraint::Length(text_height)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(layout[1]);
        Block::new().bg(SCHEME.surface).render(layout[1], buf);

        // if
        let spans = if self.cursor_pos < self.current_input.len() {
            let chars = self.current_input.chars().collect::<Vec<char>>();
            // let odkjlf: String = chars[0..2].iter().collect();
            // let around = self.current_input.split_at(self.cursor_pos);
            vec![
                Span::raw(chars[..self.cursor_pos].iter().collect::<String>()),
                Span::styled(
                    chars[self.cursor_pos].to_string(),
                    Style::new().underlined(),
                ),
                Span::raw(chars[self.cursor_pos + 1..].iter().collect::<String>()),
            ]
        } else {
            vec![
                Span::raw(&self.current_input),
                Span::styled(" ", Style::new().underlined()),
            ]
        };
        println!("{spans:?}");
        Paragraph::new(Line::from(spans))
            .wrap(Wrap { trim: true })
            .render(input_text[0], buf);
    }
}
