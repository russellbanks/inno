use std::{borrow::Cow, iter::once};

use inno::entry::File;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment,
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
    flags: Vec<String>,
}

impl<'a> Files<'a> {
    pub fn new(files: &'a [File]) -> Self {
        Self {
            files,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(files.len()),
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

    fn max_len<F>(&self, f: F, extra: usize) -> u16
    where
        F: Fn(&File) -> usize,
    {
        self.files
            .iter()
            .map(f)
            .chain(once(extra))
            .max()
            .unwrap_or(1)
            .try_into()
            .unwrap_or(u16::MAX)
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
                Cow::Owned(file.r#type().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(
                rows,
                [
                    Length(2),
                    Length(self.max_len(
                        |file| file.source().map(str::len).unwrap_or_default(),
                        SOURCE.len(),
                    )),
                    Length(self.max_len(
                        |file| file.destination().map(str::len).unwrap_or_default(),
                        DESTINATION.len(),
                    )),
                    Length(self.max_len(
                        |file| file.install_font_name().map(str::len).unwrap_or_default(),
                        ASSEMBLY_NAME.len(),
                    )),
                    Length(self.max_len(
                        |file| {
                            file.strong_assembly_name()
                                .map(str::len)
                                .unwrap_or_default()
                        },
                        ASSEMBLY_NAME.len(),
                    )),
                    Max(4),
                    Length(
                        self.flags
                            .iter()
                            .map(String::len)
                            .chain(once(FLAGS.len()))
                            .max()
                            .unwrap_or(1)
                            .try_into()
                            .unwrap_or(u16::MAX),
                    ),
                    Length(self.max_len(|file| file.r#type().as_str().len(), TYPE.len())),
                ],
            )
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
