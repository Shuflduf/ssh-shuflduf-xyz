use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use strum::{EnumCount, EnumProperty, IntoEnumIterator, VariantArray};

use crate::{
    app::{TerminalPane, border_col, make_block},
    colours::SCHEME,
    types::{Focus, Game, Games, MainPane, ServerState, Wordle},
};

#[async_trait]
impl TerminalPane for Games {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        current_pane: &mut MainPane,
        focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        if let Some(game) = self.active_game {
            match game {
                Game::Wordle => {
                    self.wordle
                        .as_mut()
                        .unwrap()
                        .handle_key(key_event, current_pane, focus, server_state, needs_redraw)
                        .await;
                }
                _ => todo!(),
            }
        } else {
            match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.focused_game = Game::VARIANTS[(Game::VARIANTS
                        .iter()
                        .position(|&item| item == self.focused_game)
                        .unwrap()
                        + Game::COUNT
                        - 1)
                        % Game::COUNT];
                    *needs_redraw = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.focused_game = Game::VARIANTS[(Game::VARIANTS
                        .iter()
                        .position(|&item| item == self.focused_game)
                        .unwrap()
                        + 1)
                        % Game::COUNT];
                    *needs_redraw = true;
                }
                KeyCode::Enter => {
                    match self.focused_game {
                        Game::Wordle => {
                            if self.wordle.is_none() {
                                self.wordle = Some(Wordle::make_game());
                            }
                        }
                        _ => todo!(),
                    }
                    self.active_game = Some(self.focused_game);
                    *needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidget for &Games {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        match self.active_game {
            Some(Game::Wordle) => self.wordle.as_ref().unwrap().render(area, buf, focus),
            Some(_) => todo!(),
            None => render_game_list(self, area, buf, focus),
        }
    }
}

fn render_game_list(games: &Games, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
    let list = Layout::vertical([Constraint::Length(3); Game::COUNT]).margin(2);
    let areas = area.layout_vec(&list);

    make_block(*focus == Focus::Pane, 2, "Games").render(area, buf);

    for (i, game) in Game::iter().enumerate() {
        Paragraph::new(Line::from(vec![
            format!(" {:07} ", game.to_string())
                .fg(SCHEME.accent)
                .bold(),
            game.get_str("Description")
                .unwrap()
                .fg(SCHEME.text_secondary)
                .italic(),
        ]))
        .block(
            Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::new().fg(border_col(games.focused_game == game))),
        )
        .render(areas[i], buf);
    }
}
