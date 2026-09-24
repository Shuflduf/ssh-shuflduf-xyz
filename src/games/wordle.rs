use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
        if code == KeyCode::Char('r') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
            *self = Wordle::make_game();
            *needs_redraw = true;
            return;
        }
        if self.guesses.last() == Some(&self.correct_word)
            || self.guesses.len() > GUESS_COUNT.into()
        {
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
            .title_bottom(key_label("Ctrl+R", "Retry").centered())
            .render(area, buf);

        let game_area = Rect::new(0, 0, WORD_LENGTH * 3, GUESS_COUNT * 3);
        let keyboard_area = Rect::new(
            game_area.right() + 6,
            0,
            KEYBOARD[0].len() as u16 * 3,
            KEYBOARD.len() as u16 * 3,
        );
        let combined_area = game_area.union(keyboard_area);
        let combined = area.centered(
            Constraint::Length(combined_area.width),
            Constraint::Length(combined_area.height),
        );

        let layout = Layout::horizontal([
            Constraint::Length(game_area.width),
            Constraint::Length(keyboard_area.width),
        ])
        .flex(Flex::SpaceBetween)
        .split(combined);

        let right_side = Layout::vertical([
            Constraint::Length(keyboard_area.height),
            Constraint::Length(3),
        ])
        .flex(Flex::SpaceEvenly)
        .split(layout[1]);
        let keyboard_layout =
            Layout::vertical([Constraint::Length(3); KEYBOARD.len()]).split(right_side[0]);
        let correct_word_layout =
            Layout::horizontal([Constraint::Length(3); WORD_LENGTH as usize]).split(right_side[1]);

        let rows = Layout::vertical([Constraint::Length(3); GUESS_COUNT as usize]).split(layout[0]);
        for (row_idx, row_area) in rows.iter().enumerate() {
            let cols =
                Layout::horizontal([Constraint::Length(3); WORD_LENGTH as usize]).split(*row_area);
            let letters = if let Some(guess) = self.guesses.get(row_idx) {
                let horse = row_idx == GUESS_COUNT as usize - 1
                    && self.guesses == vec!["horse"].repeat(GUESS_COUNT.into());

                let mut remaining_letters =
                    self.correct_word.clone().chars().collect::<Vec<char>>();

                guess
                    .chars()
                    .enumerate()
                    .map(|(pos, c)| {
                        (
                            c,
                            if horse {
                                (SCHEME.wordle_correct, SCHEME.surface_secondary)
                            } else {
                                self.color_at(pos, c, &mut remaining_letters)
                            },
                        )
                    })
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

        for (row_idx, row_area) in keyboard_layout.iter().enumerate() {
            let cols = Layout::horizontal(vec![Constraint::Length(3); KEYBOARD[row_idx].len()])
                .split(*row_area);
            let word = if self.horse() {
                "horse"
            } else {
                &self.correct_word
            };
            let letters: Vec<(char, (Color, Color))> = {
                KEYBOARD[row_idx]
                    .iter()
                    .map(|&c| (c, self.keyboard_colour(c, word)))
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

        if self.guesses.len() == GUESS_COUNT.into()
            && self.guesses.last() != Some(&self.correct_word)
        {
            let word = if self.horse() {
                "horse"
            } else {
                &self.correct_word
            };
            for (i, c) in word.chars().enumerate() {
                Wordle::letter_cell(c, SCHEME.wordle_correct, SCHEME.surface_secondary)
                    .render(correct_word_layout[i], buf);
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

    fn color_at(&self, pos: usize, c: char, remaining: &mut [char]) -> (Color, Color) {
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

    fn keyboard_colour(&self, c: char, word: &str) -> (Color, Color) {
        for guess in self.guesses.clone() {
            for (i, guess_c) in guess.chars().enumerate() {
                if c == guess_c && word.chars().collect::<Vec<char>>()[i] == guess_c {
                    return (SCHEME.wordle_correct, SCHEME.surface_secondary);
                }
            }
        }

        for guess in self.guesses.clone() {
            for guess_c in guess.chars() {
                if c == guess_c && word.chars().collect::<Vec<char>>().contains(&guess_c) {
                    return (SCHEME.wordle_hint, SCHEME.surface_secondary);
                }
            }
        }

        for guess in self.guesses.clone() {
            if guess.chars().collect::<Vec<char>>().contains(&c) {
                return (SCHEME.wordle_incorrect, SCHEME.text);
            }
        }

        (SCHEME.surface_secondary, SCHEME.text)
    }

    fn horse(&self) -> bool {
        self.guesses == vec!["horse"].repeat(GUESS_COUNT.into())
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
