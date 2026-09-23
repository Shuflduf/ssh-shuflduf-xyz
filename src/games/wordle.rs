use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{app::make_block, types::Focus};

pub fn render(area: Rect, buf: &mut Buffer, focus: &mut Focus) {
    make_block(*focus == Focus::Pane, 2, "Wordle").render(area, buf);
}
