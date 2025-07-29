use std::borrow::Cow;

use inno::entry::Component;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
        Constraint::{Fill, Length},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::emoji::Emoji;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Components<'a> {
    components: &'a [Component],
    state: TableState,
    scroll_state: ScrollbarState,
}

impl<'a> Components<'a> {
    pub fn new(components: &'a [Component]) -> Self {
        Self {
            components,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(components.len()),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.components.len())
            .map(|index| index + 1)
            .unwrap_or_default();

        self.state.select(Some(index));
        self.scroll_state = self.scroll_state.position(index);
    }

    pub fn previous_row(&mut self) {
        let index = self
            .state
            .selected()
            .map(|index| index.saturating_sub(1))
            .unwrap_or_default();

        self.state.select(Some(index));
        self.scroll_state = self.scroll_state.position(index);
    }
}

impl Widget for &mut Components<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([Fill(1), Length(1)]).split(area);

        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            layout[1],
            buf,
            &mut self.scroll_state,
        );

        let rows = self
            .components
            .iter()
            .enumerate()
            .map(|(index, component)| {
                Row::new([
                    Cow::Owned((index + 1).to_string()),
                    Cow::Borrowed(component.name().unwrap_or_default()),
                    Cow::Borrowed(component.description().unwrap_or_default()),
                    Cow::Borrowed(component.types().unwrap_or_default()),
                    Cow::Borrowed(component.languages().unwrap_or_default()),
                    Cow::Borrowed(component.check().unwrap_or_default()),
                    Cow::Owned(component.extra_disk_space_required().to_string()),
                    Cow::Owned(component.level().to_string()),
                    Cow::Borrowed(component.used().emoji()),
                    Cow::Owned(component.flags().to_string()),
                    Cow::Owned(component.size().to_string()),
                ])
            });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(4),
                    Length(11),
                    Length(5),
                    Length(9),
                    Length(5),
                    Length(25),
                    Length(5),
                    Length(4),
                    Length(5),
                    Length(4),
                ],
            )
            .header(
                Row::new([
                    "#",
                    "Name",
                    "Description",
                    "Types",
                    "Languages",
                    "Check",
                    "Extra disk space required",
                    "Level",
                    "Used",
                    "Flags",
                    "Size",
                ])
                .style(Style::new().add_modifier(Modifier::BOLD)),
            )
            .column_spacing(2)
            .block(
                Block::bordered()
                    .title("Components")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::proportional(1)),
            )
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400)),
            layout[0],
            buf,
            &mut self.state,
        );
    }
}
