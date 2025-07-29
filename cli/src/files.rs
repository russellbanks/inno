use std::borrow::Cow;

use inno::entry::File;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::constraint::{flags_constraint, int_constraint, ints_constraint, strings_constraint};

const SOURCE: &str = "Source";
const DESTINATION: &str = "Destination";
const FONT_NAME: &str = "Font name";
const ASSEMBLY_NAME: &str = "Assembly name";
const SIZE: &str = "Size";
const FLAGS: &str = "Flags";
const TYPE: &str = "Type";

const HEADERS: [&str; 8] = [
    "#",
    SOURCE,
    DESTINATION,
    FONT_NAME,
    ASSEMBLY_NAME,
    SIZE,
    FLAGS,
    TYPE,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Files<'a> {
    files: &'a [File],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 8],
    flags: Vec<String>,
}

impl<'a> Files<'a> {
    pub fn new(files: &'a [File]) -> Self {
        Self {
            files,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(files.len()),
            constraints: constraints(files),
            flags: files.iter().map(|file| file.flags().to_string()).collect(),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.files.len())
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

    /// Returns `true` if there are no file entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Widget for &mut Files<'_> {
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

        let rows = self.files.iter().enumerate().map(|(index, file)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(file.source().unwrap_or_default()),
                Cow::Borrowed(file.destination().unwrap_or_default()),
                Cow::Borrowed(file.install_font_name().unwrap_or_default()),
                Cow::Borrowed(file.strong_assembly_name().unwrap_or_default()),
                Cow::Owned(file.external_size().to_string()),
                Cow::Borrowed(&self.flags[index]),
                Cow::Borrowed(file.r#type().as_str()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::new().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Files")
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

fn constraints(files: &[File]) -> [Constraint; 8] {
    [
        Length(int_constraint(files.len())),
        Max(strings_constraint(files.iter().map(File::source), SOURCE)),
        Length(strings_constraint(
            files.iter().map(File::destination),
            DESTINATION,
        )),
        Max(strings_constraint(
            files.iter().map(File::install_font_name),
            FONT_NAME,
        )),
        Max(strings_constraint(
            files.iter().map(File::strong_assembly_name),
            ASSEMBLY_NAME,
        )),
        Max(ints_constraint(files.iter().map(File::external_size), SIZE)),
        Max(flags_constraint(files.iter().map(File::flags), FLAGS)),
        Max(strings_constraint(
            files.iter().map(|file| file.r#type().as_str()),
            TYPE,
        )),
    ]
}
