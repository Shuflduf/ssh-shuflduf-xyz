use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Focus, MainPane, ServerState, Tetris},
};

const BOARD_SIZE: (i8, i8) = (10, 20);
const GRAVITY_TIME: u8 = 50;

#[async_trait]
impl TerminalPane for Tetris {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        match key_event.code {
            KeyCode::Char('a') => *needs_redraw = self.try_move((-1, 0)),
            KeyCode::Char('d') => *needs_redraw = self.try_move((1, 0)),
            KeyCode::Left => *needs_redraw = self.try_rotate(3),
            KeyCode::Right => *needs_redraw = self.try_rotate(1),
            _ => {}
        }
    }

    async fn tick(&mut self, needs_redraw: &mut bool) {
        self.gravity_timer -= 1;
        if self.gravity_timer == 0 {
            *needs_redraw = self.try_move((0, 1));
            self.gravity_timer = GRAVITY_TIME;
        }
    }
}

impl StatefulWidget for &Tetris {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Tetris").centered())
            .title_top(key_label("Esc", "Go Back").left_aligned())
            .title_bottom(key_label("Ctrl+R", "Retry").centered())
            .render(area, buf);

        let area = area.centered(
            Constraint::Length(BOARD_SIZE.0 as u16 * 2),
            Constraint::Length(BOARD_SIZE.1 as u16),
        );
        Block::new().bg(SCHEME.surface).render(area, buf);

        for (y, row_area) in Layout::vertical([Constraint::Length(1); BOARD_SIZE.1 as usize])
            .split(area)
            .iter()
            .enumerate()
        {
            for (x, cell_area) in Layout::horizontal([Constraint::Length(2); BOARD_SIZE.0 as usize])
                .split(*row_area)
                .iter()
                .enumerate()
            {
                let pos = (x as i8, y as i8);
                let piece_pos = (pos.0 - self.pos.0, pos.1 - self.pos.1);
                if self.table.pieces[self.index as usize][self.rot as usize].contains(&piece_pos) {
                    Block::new()
                        .bg(Tetris::get_col(self.index))
                        .render(*cell_area, buf);
                }
            }
        }
    }
}

impl Tetris {
    pub fn make_game() -> Tetris {
        Tetris {
            index: 1,
            pos: (3, 0),
            rot: 0,
            gravity_timer: GRAVITY_TIME,
            table: serde_json::from_str(include_str!("tetris_srs.json")).unwrap(),
        }
    }

    fn try_move(&mut self, dir: (i8, i8)) -> bool {
        let test_pos = (self.pos.0 + dir.0, self.pos.1 + dir.1);
        for tile in &self.table.pieces[self.index as usize][self.rot as usize] {
            let tile_pos = (test_pos.0 + tile.0, test_pos.1 + tile.1);
            if tile_pos.0 < 0
                || tile_pos.1 < 0
                || tile_pos.0 >= BOARD_SIZE.0
                || tile_pos.1 >= BOARD_SIZE.1
            {
                return false;
            }
        }
        self.pos = test_pos;
        true
    }

    fn try_rotate(&mut self, dir: u8) -> bool {
        let test_rot = (self.rot + dir) % 4;
        for tile in &self.table.pieces[self.index as usize][test_rot as usize] {
            let tile_pos = (self.pos.0 + tile.0, self.pos.1 + tile.1);
            if tile_pos.0 < 0
                || tile_pos.1 < 0
                || tile_pos.0 >= BOARD_SIZE.0
                || tile_pos.1 >= BOARD_SIZE.1
            {
                return false;
            }
        }
        self.rot = test_rot;
        true
    }

    fn get_col(index: u8) -> Color {
        match index {
            0 => SCHEME.tetris_red,
            1 => SCHEME.tetris_orange,
            2 => SCHEME.tetris_yellow,
            3 => SCHEME.tetris_green,
            4 => SCHEME.tetris_cyan,
            5 => SCHEME.tetris_blue,
            6 => SCHEME.tetris_pink,
            _ => unreachable!(),
        }
    }
}
