use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Focus, MainPane, ServerState, Wordle},
};

const WORD_LENGTH: u16 = 5;
const GUESS_COUNT: u16 = 6;
const WORDLE_ALLOWED: &str = include_str!("wordle_allowed.txt");
const WORDLE_ANSWERS: &str = include_str!("wordle_answers.txt");
const KEYBOARD: &[&[char]] = &[
    &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
    &['z', 'x', 'c', 'v', 'b', 'n', 'm'],
];

#[async_trait]
impl TerminalPane for Wordle {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        let code = key_event.code;
        if self.guesses.last() == Some(&self.correct_word) {
            return;
        }
        if (KeyCode::Char('a')..=KeyCode::Char('z')).contains(&code) {
            if self.current_guess.len() < 5 {
                self.current_guess += &code.as_char().unwrap().to_string();
                *needs_redraw = true;
            }
        } else {
            match code {
                KeyCode::Backspace | KeyCode::Delete => {
                    self.current_guess.pop();
                    *needs_redraw = true;
                }
                KeyCode::Enter => {
                    if self.current_guess.len() == WORD_LENGTH as usize
                        && Wordle::allowed_list().contains(&self.current_guess.as_str().trim())
                    {
                        self.guesses.push(self.current_guess.clone());
                        self.current_guess = String::new();
                    }
                    *needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidget for &Wordle {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Wordle").centered())
            .title_top(key_label("Esc", "Go Back").left_aligned())
            .render(area, buf);

        let game_area = area.centered(
            Constraint::Length(WORD_LENGTH * 3),
            Constraint::Length(GUESS_COUNT * 3),
        );
        let layout = Layout::horizontal([
            Constraint::Length(game_area.width),
            Constraint::Length(KEYBOARD[0].len() as u16 * 3),
        ])
        .flex(Flex::SpaceEvenly)
        .split(area);

        let rows = Layout::vertical([Constraint::Length(3); GUESS_COUNT as usize]).split(layout[0]);
        for (row_idx, row_area) in rows.iter().enumerate() {
            let cols =
                Layout::horizontal([Constraint::Length(3); WORD_LENGTH as usize]).split(*row_area);
            let letters = if let Some(guess) = self.guesses.get(row_idx) {
                guess
                    .chars()
                    .enumerate()
                    .map(|(pos, c)| (c, self.color_at(pos, c)))
                    .collect()
            } else if row_idx == self.guesses.len() {
                self.current_guess
                    .chars()
                    .map(|c| (c, (SCHEME.surface_secondary, SCHEME.text)))
                    .collect()
            } else {
                vec![]
            };

            for (col_idx, col_area) in cols.iter().enumerate() {
                let (c, (bg, fg)) = letters
                    .get(col_idx)
                    .copied()
                    .unwrap_or((' ', (SCHEME.surface, SCHEME.text)));
                Wordle::letter_cell(c, bg, fg).render(*col_area, buf);
            }
        }

        let keyboard_layout =
            Layout::vertical([Constraint::Length(3); KEYBOARD.len()]).split(layout[1]);
        for (row_idx, row_area) in keyboard_layout.iter().enumerate() {
            let cols = Layout::horizontal(vec![Constraint::Length(3); KEYBOARD[row_idx].len()])
                .split(*row_area);
            let letters: Vec<(char, (Color, Color))> = {
                KEYBOARD[row_idx]
                    .iter()
                    .map(|&c| (c, self.keyboard_colour(c)))
                    .collect()
            };

            for (col_idx, col_area) in cols.iter().enumerate() {
                let (c, (bg, fg)) = letters
                    .get(col_idx)
                    .copied()
                    .unwrap_or((' ', (SCHEME.surface, SCHEME.text)));
                Wordle::letter_cell(c, bg, fg).render(*col_area, buf);
            }
        }
    }
}

impl Wordle {
    pub fn make_game() -> Self {
        let words = Wordle::answer_list();
        Self {
            correct_word: words[rand::random_range(0..words.len())].to_string(),
            // correct_word: "horse".to_string(),
            guesses: vec![],
            current_guess: String::new(),
        }
    }

    fn color_at(&self, pos: usize, c: char) -> (Color, Color) {
        let mut remaining: Vec<char> = self.correct_word.clone().chars().collect();
        if remaining[pos] == c {
            remaining[pos] = ' ';
            (SCHEME.wordle_correct, SCHEME.surface_secondary)
        } else if remaining.contains(&c) {
            remaining[pos] = ' ';
            (SCHEME.wordle_hint, SCHEME.surface_secondary)
        } else {
            (SCHEME.wordle_incorrect, SCHEME.text)
        }
    }

    fn keyboard_colour(&self, c: char) -> (Color, Color) {
        for guess in self.guesses.clone() {
            for (i, guess_c) in guess.chars().enumerate() {
                if c == guess_c && self.correct_word.chars().collect::<Vec<char>>()[i] == guess_c {
                    return (SCHEME.wordle_correct, SCHEME.surface_secondary);
                }
            }
        }

        for guess in self.guesses.clone() {
            for guess_c in guess.chars() {
                if c == guess_c
                    && self
                        .correct_word
                        .chars()
                        .collect::<Vec<char>>()
                        .contains(&guess_c)
                {
                    return (SCHEME.wordle_hint, SCHEME.surface_secondary);
                }
            }
        }

        for guess in self.guesses.clone() {
            for guess_c in guess.chars() {
                if c == guess_c {
                    return (SCHEME.wordle_incorrect, SCHEME.text);
                }
            }
        }

        (SCHEME.surface, SCHEME.text)
    }

    fn letter_cell(c: char, bg: Color, fg: Color) -> Paragraph<'static> {
        Paragraph::new(vec![
            Line::raw("   "),
            Line::raw(format!(" {} ", c.to_ascii_uppercase())),
            Line::raw("   "),
        ])
        .bg(bg)
        .fg(fg)
    }

    fn allowed_list() -> Vec<&'static str> {
        WORDLE_ALLOWED.lines().collect()
    }
    fn answer_list() -> Vec<&'static str> {
        WORDLE_ANSWERS.lines().collect()
    }
}
