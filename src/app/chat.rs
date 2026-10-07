use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    types::{Content, Focus, ServerMessage, ServerState},
};

#[derive(Default)]
pub struct Chat {
    config_open: bool,
    messages: Vec<String>,
    current_input: String,
    cursor_pos: usize,
}

#[async_trait]
impl TerminalPane for Chat {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut Content,
        _focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        if key_event.code == KeyCode::Char('e')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.toggle_config();
            *needs_redraw = true;
            return;
        }
        if self.config_open {
            if key_event.code == KeyCode::Esc {
                self.toggle_config();
                *needs_redraw = true;
            }
        } else {
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
                KeyCode::Delete => {
                    if self.cursor_pos < self.current_input.len() {
                        self.current_input.remove(self.cursor_pos);
                        // self.cursor_pos -= 1;
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
                KeyCode::Enter if !self.current_input.is_empty() => {
                    self.cursor_pos = 0;
                    let _ = server_state
                        .broadcast_sender
                        .send(ServerMessage::ChatMessage(self.current_input.clone()));
                    self.current_input = String::new();
                    *needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidget for &Chat {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane && !self.config_open)
            .title_top(
                if !self.config_open {
                    key_label("2", "Chat")
                } else {
                    Line::from(" Chat ").bold()
                }
                .centered(),
            )
            .render(area, buf);

        let line_length = (area.width - 8) as f32;
        let text_height = (((self.current_input.len() as f32) / line_length).ceil() as u16).max(1);

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(text_height + 2)])
            .spacing(1)
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(area);

        let messages_layout =
            Layout::vertical([Constraint::Length(3)].repeat(self.messages.len())).split(layout[0]);

        for (i, message) in self.messages.iter().enumerate() {
            Paragraph::new(message.clone()).render(messages_layout[i], buf);
        }

        let input_text = Layout::vertical([Constraint::Length(text_height)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(layout[1]);
        Block::new().bg(SCHEME.surface).render(layout[1], buf);

        let spans = if self.cursor_pos < self.current_input.len() {
            let chars = self.current_input.chars().collect::<Vec<char>>();
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
        Paragraph::new(Line::from(spans))
            .wrap(Wrap { trim: true })
            .render(input_text[0], buf);

        if self.config_open {
            make_block(*focus == Focus::Pane)
                .title_top(key_label("2", "Config").centered())
                .title_bottom(key_label("Esc", "Close").centered())
                .render(
                    area.centered(Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)),
                    buf,
                );
        }
    }
}

impl Chat {
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    fn toggle_config(&mut self) {
        self.config_open = !self.config_open;
    }
}
