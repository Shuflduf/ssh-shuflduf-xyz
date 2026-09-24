use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Focus, MainPane, ServerState, Snake},
};

const BOARD_SIZE: i8 = 13;

#[async_trait]
impl TerminalPane for Snake {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        _server_state: &ServerState,
        _needs_redraw: &mut bool,
    ) {
        match key_event.code {
            KeyCode::Char('w') => self.current_dir = (0, -1),
            _ => {}
        }
    }
}

impl StatefulWidget for &Snake {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Snake").centered())
            .title_top(key_label("Esc", "Go Back").left_aligned())
            .title_bottom(key_label("Ctrl+R", "Retry").centered())
            .render(area, buf);

        let area = area.centered(
            Constraint::Length(BOARD_SIZE as u16 * 2),
            Constraint::Length(BOARD_SIZE as u16),
        );
        Block::new().bg(SCHEME.surface).render(area, buf);
        for (y, row_area) in Layout::vertical([Constraint::Length(1); BOARD_SIZE as usize])
            .split(area)
            .iter()
            .enumerate()
        {
            for (x, cell_area) in Layout::horizontal([Constraint::Length(2); BOARD_SIZE as usize])
                .split(*row_area)
                .iter()
                .enumerate()
            {
                let pos = (x as i8, y as i8);
                if self.tiles.contains(&pos) {
                    Block::new().bg(SCHEME.snake_body).render(*cell_area, buf);
                }
                if self.tiles[0] == pos {
                    Block::new().bg(SCHEME.snake_head).render(*cell_area, buf);
                }

                if self.fruit_pos == pos {
                    Block::new().bg(SCHEME.snake_fruit).render(*cell_area, buf);
                }
            }
        }
    }
}

impl Snake {
    pub fn make_game() -> Snake {
        Snake {
            current_dir: (1, 0),
            tiles: vec![(3, 6), (2, 6), (1, 6)],
            fruit_pos: (8, 6),
        }
    }
}
