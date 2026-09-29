use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::random_range;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    extra::RemoveFirst,
    types::{Focus, MainPane, ServerState},
};

const BOARD_SIZE: i8 = 13;
const MOVE_TIME: u8 = 20;

pub struct Snake {
    current_dir: (i8, i8),
    queued_dirs: Vec<(i8, i8)>,
    tiles: Vec<(i8, i8)>,
    fruit_pos: (i8, i8),
    move_timer: u8,
}
pub struct SnakeScheme {
    pub body: Color,
    pub head: Color,
    pub fruit: Color,
}

#[async_trait]
impl TerminalPane for Snake {
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
            *self = Snake::make_game();
            *needs_redraw = true;
            return;
        }
        match key_event.code {
            KeyCode::Char('w') => self.queue_next_dir((0, -1)),
            KeyCode::Char('a') => self.queue_next_dir((-1, 0)),
            KeyCode::Char('s') => self.queue_next_dir((0, 1)),
            KeyCode::Char('d') => self.queue_next_dir((1, 0)),
            _ => {}
        }
    }

    async fn tick(&mut self, needs_redraw: &mut bool) {
        if !self.alive() {
            return;
        }
        self.move_timer -= 1;
        if self.move_timer == 0 {
            if let Some(next) = self.queued_dirs.pop_first() {
                self.current_dir = next;
            }
            self.move_timer = MOVE_TIME;
            self.proceed();
            // if self.alive() {
            *needs_redraw = true;
            // }
        }
    }
}

impl StatefulWidget for &Snake {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Snake").centered())
            .title_top(key_label("Esc", "Go Back").left_aligned())
            .title_bottom(
                Line::from_iter(
                    key_label("Ctrl+R", "Retry")
                        .spans
                        .into_iter()
                        .chain(key_label("WASD", "Move").spans),
                )
                .centered(),
            )
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
                    Block::new().bg(SCHEME.snake.body).render(*cell_area, buf);
                }
                if self.tiles[0] == pos {
                    Block::new().bg(SCHEME.snake.head).render(*cell_area, buf);
                }

                if self.fruit_pos == pos {
                    Block::new().bg(SCHEME.snake.fruit).render(*cell_area, buf);
                }
            }
        }
    }
}

impl Snake {
    pub fn make_game() -> Snake {
        Snake {
            current_dir: (1, 0),
            queued_dirs: vec![],
            tiles: vec![(3, 6), (2, 6), (1, 6)],
            fruit_pos: (8, 6),
            move_timer: MOVE_TIME,
        }
    }

    fn proceed(&mut self) {
        let last_pos = *self.tiles.last().unwrap();
        for i in (1..self.tiles.len()).rev() {
            self.tiles[i] = self.tiles[i - 1]
        }
        self.tiles[0].0 += self.current_dir.0;
        self.tiles[0].1 += self.current_dir.1;

        if self.tiles[0] == self.fruit_pos {
            self.tiles.push(last_pos);
            self.place_new_fruit();
        }
    }

    fn place_new_fruit(&mut self) {
        loop {
            let test_pos = (random_range(0..BOARD_SIZE), random_range(0..BOARD_SIZE));
            if !self.tiles.contains(&test_pos) {
                self.fruit_pos = test_pos;
                break;
            }
        }
    }

    fn queue_next_dir(&mut self, dir: (i8, i8)) {
        let opp_dir = match dir {
            (-1, 0) => (1, 0),
            (1, 0) => (-1, 0),
            (0, 1) => (0, -1),
            (0, -1) => (0, 1),
            _ => unreachable!(),
        };
        if &opp_dir != self.queued_dirs.last().unwrap_or(&self.current_dir) {
            self.queued_dirs.push(dir);
        }
    }

    fn alive(&self) -> bool {
        let head = self.tiles[0];
        if head.0 > BOARD_SIZE || head.1 > BOARD_SIZE || head.0 < 0 || head.1 < 0 {
            return false;
        }
        if self.tiles[1..].contains(&head) {
            return false;
        }
        true
    }
}
