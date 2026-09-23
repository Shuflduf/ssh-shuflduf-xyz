use async_trait::async_trait;
use crossterm::event::KeyCode;
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
    games::wordle,
    types::{Focus, Game, Games, MainPane, ServerState},
};

#[async_trait]
impl TerminalPane for Games {
    async fn handle_key(
        &mut self,
        key_code: KeyCode,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match key_code {
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
                self.active_game = Some(self.focused_game);
                *needs_redraw = true;
            }
            _ => {}
        }
    }
}

impl StatefulWidget for &Games {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        match self.active_game {
            Some(Game::Wordle) => wordle::render(area, buf, focus),
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
            format!(" {:07} ", game.to_string()).blue().bold(),
            game.get_str("Description").unwrap().dim().italic(),
        ]))
        .block(
            Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::new().fg(border_col(games.focused_game == game))),
        )
        .render(areas[i], buf);
    }
}
