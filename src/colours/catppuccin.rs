use ratatui::style::Color;

use crate::{
    games::{snake::SnakeScheme, tetris::TetrisScheme, wordle::WordleScheme},
    types::ColourScheme,
};

pub const CATPPUCCIN: ColourScheme = ColourScheme {
    base: Color::Rgb(30, 30, 46),
    surface: Color::Rgb(49, 50, 68),
    surface_secondary: Color::Rgb(69, 71, 90),
    text: Color::Rgb(205, 214, 244),
    text_secondary: Color::Rgb(166, 173, 200),
    keys: Color::Rgb(137, 180, 250),
    accent: Color::Rgb(203, 166, 247),

    wordle: WordleScheme {
        correct: Color::Rgb(166, 227, 161),
        correct_text: Color::Rgb(30, 30, 46),
        hint: Color::Rgb(249, 226, 175),
        hint_text: Color::Rgb(30, 30, 46),
        incorrect: Color::Rgb(49, 50, 68),
        incorrect_text: Color::Rgb(205, 214, 244),
        unknown: Color::Rgb(69, 71, 90),
        unknown_text: Color::Rgb(205, 214, 244),
    },

    snake: SnakeScheme {
        body: Color::Rgb(166, 227, 161),
        head: Color::Rgb(137, 180, 250),
        fruit: Color::Rgb(243, 139, 168),
    },

    tetris: TetrisScheme {
        red: Color::Rgb(243, 139, 168),
        orange: Color::Rgb(250, 179, 135),
        yellow: Color::Rgb(249, 226, 175),
        green: Color::Rgb(166, 227, 161),
        cyan: Color::Rgb(137, 220, 235),
        blue: Color::Rgb(137, 180, 250),
        pink: Color::Rgb(245, 194, 231),
    },
};
