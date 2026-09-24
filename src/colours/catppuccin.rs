use ratatui::style::Color;

use crate::types::ColourScheme;

pub const CATPPUCCIN: ColourScheme = ColourScheme {
    base: Color::Rgb(30, 30, 46),
    surface: Color::Rgb(49, 50, 68),
    surface_secondary: Color::Rgb(69, 71, 90),
    text: Color::Rgb(205, 214, 244),
    text_secondary: Color::Rgb(166, 173, 200),
    keys: Color::Rgb(137, 180, 250),
    accent: Color::Rgb(203, 166, 247),

    wordle_correct: Color::Rgb(166, 227, 161),
    wordle_correct_text: Color::Rgb(30, 30, 46),
    wordle_hint: Color::Rgb(249, 226, 175),
    wordle_hint_text: Color::Rgb(30, 30, 46),
    wordle_incorrect: Color::Rgb(49, 50, 68),
    wordle_incorrect_text: Color::Rgb(205, 214, 244),
    wordle_unknown: Color::Rgb(69, 71, 90),
    wordle_unknown_text: Color::Rgb(205, 214, 244),

    snake_body: Color::Rgb(166, 227, 161),
    snake_head: Color::Rgb(137, 180, 250),
    snake_fruit: Color::Rgb(243, 139, 168),
};
