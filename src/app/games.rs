use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    app::make_block,
    types::{Focus, Game, Games},
};

impl StatefulWidget for &Games {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let list = Layout::vertical([Constraint::Length(10); Game::COUNT]).margin(2);
        let areas = area.layout_vec(&list);

        make_block(*focus == Focus::Pane, 2, "Games").render(area, buf);

        for (i, game) in Game::iter().enumerate() {
            Paragraph::new(game.to_string())
                .block(Block::bordered().border_set(border::ROUNDED))
                .render(areas[i], buf);
        }
    }
}
