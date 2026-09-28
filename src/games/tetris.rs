use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::seq::SliceRandom;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, StatefulWidget, Widget},
};
use russh::keys::ssh_key::sec1::der::Tag::TeletexString;
use serde::Deserialize;

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Focus, MainPane, ServerState},
};

pub const BOARD_SIZE: (i8, i8) = (10, 20);
const GRAVITY_TIME: u8 = 50;

#[derive(Deserialize)]
struct SRSTable {
    pieces: Vec<Vec<Vec<(i8, i8)>>>,
    kicks: Vec<Vec<(i8, i8)>>,
    kicks_i: Vec<Vec<(i8, i8)>>,
}

pub struct Tetris {
    index: usize,
    pos: (i8, i8),
    rot: usize,
    gravity_timer: u8,
    bag: Vec<usize>,
    board: [[Option<usize>; 20]; 10],
    table: SRSTable,
}

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
        if key_event.code == KeyCode::Char('r')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            *self = Tetris::make_game();
            *needs_redraw = true;
            return;
        }
        match key_event.code {
            KeyCode::Char('a') => *needs_redraw = self.try_move((-1, 0)),
            KeyCode::Char('d') => *needs_redraw = self.try_move((1, 0)),
            KeyCode::Char('w') => *needs_redraw = self.apply_gravity(),
            KeyCode::Char('s') => *needs_redraw = self.hard_drop(),
            KeyCode::Left => *needs_redraw = self.try_rotate(3),
            KeyCode::Right => *needs_redraw = self.try_rotate(1),
            _ => {}
        }
    }

    async fn tick(&mut self, needs_redraw: &mut bool) {
        self.gravity_timer -= 1;
        if self.gravity_timer == 0 {
            *needs_redraw = self.apply_gravity();
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
                if self.table.pieces[self.index][self.rot].contains(&piece_pos) {
                    Block::new()
                        .bg(Tetris::get_col(self.index))
                        .render(*cell_area, buf);
                } else if let Some(idx) = self.board[pos.0 as usize][pos.1 as usize] {
                    Block::new()
                        .bg(Tetris::get_col(idx))
                        .render(*cell_area, buf);
                }
            }
        }
    }
}

impl Tetris {
    pub fn make_game() -> Tetris {
        let mut bag = Tetris::new_bag();
        Tetris {
            index: bag.pop().unwrap(),
            pos: (3, 0),
            rot: 0,
            gravity_timer: GRAVITY_TIME,
            bag,
            board: [[None; 20]; 10],
            table: serde_json::from_str(include_str!("tetris_srs.json")).unwrap(),
        }
    }

    fn new_bag() -> Vec<usize> {
        let mut bag = vec![0, 1, 2, 3, 4, 5, 6];
        bag.shuffle(&mut rand::rng());
        bag
    }

    fn kick_index(before: usize, after: usize) -> usize {
        if after == (before + 1) % 4 {
            return before * 2;
        } else {
            return (before * 2 + 7) % 8;
        }
    }

    fn try_move(&mut self, dir: (i8, i8)) -> bool {
        let test_pos = (self.pos.0 + dir.0, self.pos.1 + dir.1);
        for tile in &self.table.pieces[self.index][self.rot] {
            let tile_pos = (test_pos.0 + tile.0, test_pos.1 + tile.1);
            if tile_pos.0 < 0
                || tile_pos.1 < 0
                || tile_pos.0 >= BOARD_SIZE.0
                || tile_pos.1 >= BOARD_SIZE.1
                || self.board[tile_pos.0 as usize][tile_pos.1 as usize].is_some()
            {
                return false;
            }
        }
        self.pos = test_pos;
        true
    }

    fn try_rotate(&mut self, dir: usize) -> bool {
        let test_rot = (self.rot + dir) % 4;

        'outer: for kick in [(0, 0)]
            .into_iter()
            .chain(self.table.kicks[Tetris::kick_index(self.rot, test_rot)].clone())
        {
            println!("{kick:?}");
            for tile in &self.table.pieces[self.index][test_rot] {
                let tile_pos = (self.pos.0 + tile.0 + kick.0, self.pos.1 + tile.1 + kick.1);
                if tile_pos.0 < 0
                    || tile_pos.1 < 0
                    || tile_pos.0 >= BOARD_SIZE.0
                    || tile_pos.1 >= BOARD_SIZE.1
                    || self.board[tile_pos.0 as usize][tile_pos.1 as usize].is_some()
                {
                    continue 'outer;
                }
            }
            self.rot = test_rot;
            self.pos.0 += kick.0;
            self.pos.1 += kick.1;
            return true;
        }
        false
    }

    fn apply_gravity(&mut self) -> bool {
        self.gravity_timer = GRAVITY_TIME;
        let success = self.try_move((0, 1));
        if !success {
            self.place_piece();
            self.reset_piece();
        }
        true
    }

    fn reset_piece(&mut self) {
        self.index = self.bag.pop().unwrap_or_else(|| {
            self.bag = Tetris::new_bag();
            self.bag.pop().unwrap()
        });
        self.pos = (3, 0);
        self.rot = 0;

        let mut should_reset = false;
        for tile in &self.table.pieces[self.index][self.rot] {
            let tile_pos = (self.pos.0 + tile.0, self.pos.1 + tile.1);
            if self.board[tile_pos.0 as usize][tile_pos.1 as usize].is_some() {
                should_reset = true;
                break;
            }
        }
        if should_reset {
            *self = Tetris::make_game()
        }
    }

    fn hard_drop(&mut self) -> bool {
        while self.try_move((0, 1)) {}
        self.gravity_timer = 1;
        self.place_piece();
        true
    }

    fn place_piece(&mut self) {
        for tile in &self.table.pieces[self.index][self.rot] {
            let tile_pos = (self.pos.0 + tile.0, self.pos.1 + tile.1);
            self.board[tile_pos.0 as usize][tile_pos.1 as usize] = Some(self.index);
        }
        let full = self.full_lines();

        for line in &full {
            for y in (1..=*line).rev() {
                for x in 0..BOARD_SIZE.0 as usize {
                    self.board[x][y] = self.board[x][y - 1];
                }
            }
        }
        if !full.is_empty() {
            for x in 0..BOARD_SIZE.0 as usize {
                self.board[x][0] = None;
            }
        }
    }

    fn full_lines(&self) -> Vec<usize> {
        let mut full = vec![];
        'outer: for y in 0..BOARD_SIZE.1 as usize {
            for x in 0..BOARD_SIZE.0 as usize {
                if self.board[x][y].is_none() {
                    continue 'outer;
                }
            }
            full.push(y);
        }
        full
    }

    fn get_col(index: usize) -> Color {
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
