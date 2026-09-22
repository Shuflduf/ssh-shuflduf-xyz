use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

use crate::{app::make_block, data::GAMES_LIST, types::Games};

impl Widget for &Games {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list = Layout::vertical([Constraint::Length(10); GAMES_LIST.len()]).margin(2);
        let areas = area.layout_vec(&list);

        make_block(self.focused, 2, "Games").render(area, buf);

        for (i, &game) in GAMES_LIST.iter().enumerate() {
            Paragraph::new(game)
                .block(Block::bordered().border_set(border::ROUNDED))
                .render(areas[i], buf);
        }
    }
}
