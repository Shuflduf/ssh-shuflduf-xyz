use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::seq::SliceRandom;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};
use serde::Deserialize;

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    extra::RemoveFirst,
    types::{Focus, MainPane, ServerState},
};

pub const BOARD_SIZE: (i8, i8) = (10, 20);
const GRAVITY_TIME: u8 = 50;
const I_PIECE: usize = 4;
const NEXT_COUNT: usize = 1;

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
    next: Vec<usize>,
    board: [[Option<usize>; 20]; 10],
    table: SRSTable,
}
pub struct TetrisScheme {
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub cyan: Color,
    pub blue: Color,
    pub pink: Color,
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
            .title_bottom(
                Line::from_iter(
                    key_label("Ctrl+R", "Retry")
                        .spans
                        .into_iter()
                        .chain(key_label("AD", "󰁍 󰁔").spans)
                        .chain(key_label("W", "󰁅").spans)
                        .chain(key_label("S", "󰞖").spans)
                        .chain(key_label("󰁍 󰁔", "󱞭 󱞯").spans),
                )
                .centered(),
            )
            .render(area, buf);

        let board_rect = Rect::new(0, 0, BOARD_SIZE.0 as u16 * 2, BOARD_SIZE.1 as u16);
        let next_rect = Rect::new(
            BOARD_SIZE.0 as u16 * 2 + 6,
            0,
            12,
            NEXT_COUNT as u16 * 3 + 1,
        );
        let total_rect = board_rect.union(next_rect);

        let area = area.centered(
            Constraint::Length(total_rect.width),
            Constraint::Length(total_rect.height),
        );
        let layout = Layout::horizontal([
            Constraint::Length(board_rect.width),
            Constraint::Length(next_rect.width),
        ])
        .flex(Flex::SpaceBetween)
        .split(area);

        Block::new().bg(SCHEME.surface).render(layout[0], buf);

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
                } else if self.table.pieces[self.index][self.rot]
                    .contains(&(pos.0 - self.pos.0, pos.1 - self.ghost_y()))
                {
                    Block::new()
                        .bg(SCHEME.surface_secondary)
                        .render(*cell_area, buf);
                }
            }
        }

        Block::new().bg(SCHEME.surface).render(
            Layout::vertical([Constraint::Length(next_rect.height)]).split(layout[1])[0],
            buf,
        );

        for (piece_idx, piece_area) in Layout::vertical([Constraint::Length(2); NEXT_COUNT])
            .vertical_margin(1)
            .horizontal_margin(2)
            .spacing(1)
            .split(layout[1])
            .iter()
            .enumerate()
        {
            for (y, row_area) in Layout::vertical([Constraint::Length(1); 2])
                .split(*piece_area)
                .iter()
                .enumerate()
            {
                for (x, cell_area) in Layout::horizontal([Constraint::Length(2); 4])
                    .split(*row_area)
                    .iter()
                    .enumerate()
                {
                    if self.table.pieces[self.next[piece_idx]][0].contains(&(x as i8, y as i8)) {
                        Block::new()
                            .bg(Tetris::get_col(self.next[piece_idx]))
                            .render(*cell_area, buf);
                    }
                }
            }
        }
    }
}

impl Tetris {
    pub fn make_game() -> Tetris {
        let mut bag = Tetris::new_bag();
        let next = ((0..=NEXT_COUNT)
            .into_iter()
            .map(|_| Tetris::next_from_bag(&mut bag)))
        .collect();
        Tetris {
            index: Tetris::next_from_bag(&mut bag),
            pos: (3, 0),
            rot: 0,
            gravity_timer: GRAVITY_TIME,
            bag,
            next,
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
            before * 2
        } else {
            (before * 2 + 7) % 8
        }
    }
    fn get_col(index: usize) -> Color {
        match index {
            0 => SCHEME.tetris.red,
            1 => SCHEME.tetris.orange,
            2 => SCHEME.tetris.yellow,
            3 => SCHEME.tetris.green,
            4 => SCHEME.tetris.cyan,
            5 => SCHEME.tetris.blue,
            6 => SCHEME.tetris.pink,
            _ => unreachable!(),
        }
    }
    fn next_from_bag(bag: &mut Vec<usize>) -> usize {
        bag.pop().unwrap_or_else(|| {
            *bag = Tetris::new_bag();
            bag.pop().unwrap()
        })
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

        let kick_table = if self.index == I_PIECE {
            &self.table.kicks_i
        } else {
            &self.table.kicks
        };
        'outer: for kick in [(0, 0)]
            .into_iter()
            .chain(kick_table[Tetris::kick_index(self.rot, test_rot)].clone())
        {
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
        self.index = self.next.pop_first().unwrap();
        self.next.push(Tetris::next_from_bag(&mut self.bag));
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
        self.place_piece();
        self.reset_piece();
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

    fn ghost_y(&self) -> i8 {
        let mut current = self.pos.1;
        while {
            let mut ce = true;
            for tile in &self.table.pieces[self.index][self.rot] {
                let tile_pos = (self.pos.0 + tile.0, tile.1 + current);
                if tile_pos.0 < 0
                    || tile_pos.1 < 0
                    || tile_pos.0 >= BOARD_SIZE.0
                    || tile_pos.1 >= BOARD_SIZE.1
                    || self.board[tile_pos.0 as usize][tile_pos.1 as usize].is_some()
                {
                    ce = false;
                }
            }
            ce
        } {
            current += 1;
        }
        current - 1
    }
}
