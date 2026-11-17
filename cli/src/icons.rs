use std::borrow::Cow;

use inno::entry::Icon;
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Constraint,
        Constraint::{Fill, Length, Max},
        Flex, Layout, Rect,
    },
    prelude::{Modifier, StatefulWidget, Style, Widget},
    style::palette::tailwind::SKY,
    widgets::{
        Block, BorderType, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::constraint::{index_width, width};

const NAME: &str = "Name";
const FILENAME: &str = "Filename";
const PARAMETERS: &str = "Parameters";
const WORKING_DIRECTORY: &str = "Working Directory";
const FILE: &str = "File";
const COMMENT: &str = "Comment";
const APP_USER_MODEL_ID: &str = "App user model ID";
const APP_USER_MODEL_TOAST_ACTIVATOR_CLSID: &str = "Toast activator CLSID";
const INDEX: &str = "Index";
const SHOW_COMMAND: &str = "Show command";
const CLOSE_ON_EXIT: &str = "Close on exit";
const HOTKEY: &str = "Hotkey";
const FLAGS: &str = "Flags";

const HEADERS: [&str; 14] = [
    "#",
    NAME,
    FILENAME,
    PARAMETERS,
    WORKING_DIRECTORY,
    FILE,
    COMMENT,
    APP_USER_MODEL_ID,
    APP_USER_MODEL_TOAST_ACTIVATOR_CLSID,
    INDEX,
    SHOW_COMMAND,
    CLOSE_ON_EXIT,
    HOTKEY,
    FLAGS,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Icons<'a> {
    icons: &'a [Icon],
    state: TableState,
    scroll_state: ScrollbarState,
    constraints: [Constraint; 14],
}

impl<'a> Icons<'a> {
    pub fn new(icons: &'a [Icon]) -> Self {
        Self {
            icons,
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::new(icons.len()),
            constraints: constraints(icons),
        }
    }

    pub fn next_row(&mut self) {
        let index = self
            .state
            .selected()
            .filter(|&index| index < self.icons.len())
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

    /// Returns `true` if there are no icon entries.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }
}

impl Widget for &mut Icons<'_> {
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

        let rows = self.icons.iter().enumerate().map(|(index, icon)| {
            Row::new([
                Cow::Owned((index + 1).to_string()),
                Cow::Borrowed(icon.name().unwrap_or_default()),
                Cow::Borrowed(icon.filename().unwrap_or_default()),
                Cow::Borrowed(icon.parameters().unwrap_or_default()),
                Cow::Borrowed(icon.working_directory().unwrap_or_default()),
                Cow::Borrowed(icon.file().unwrap_or_default()),
                Cow::Borrowed(icon.comment().unwrap_or_default()),
                Cow::Borrowed(icon.app_user_model_id().unwrap_or_default()),
                Cow::Borrowed(icon.app_user_model_toast_activator_clsid()),
                Cow::Owned(icon.index().to_string()),
                Cow::Owned(icon.show_command().to_string()),
                Cow::Borrowed(icon.close_on_exit().as_str()),
                Cow::Owned(icon.hotkey().to_string()),
                Cow::Owned(icon.flags().to_string()),
            ])
        });

        StatefulWidget::render(
            Table::new(rows, self.constraints)
                .header(Row::new(HEADERS).style(Style::default().add_modifier(Modifier::BOLD)))
                .column_spacing(2)
                .block(
                    Block::bordered()
                        .title("Icons")
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded)
                        .padding(Padding::proportional(1)),
                )
                .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED).fg(SKY.c400))
                .flex(Flex::Legacy),
            layout[0],
            buf,
            &mut self.state,
        );
    }
}

fn constraints(icons: &[Icon]) -> [Constraint; 14] {
    [
        Max(index_width(icons.len())),
        Max(width(
            NAME,
            icons.iter().map(|icon| icon.name().unwrap_or_default()),
        )),
        Max(width(
            FILENAME,
            icons.iter().map(|icon| icon.filename().unwrap_or_default()),
        )),
        Max(width(
            PARAMETERS,
            icons
                .iter()
                .map(|icon| icon.parameters().unwrap_or_default()),
        )),
        Max(width(
            WORKING_DIRECTORY,
            icons
                .iter()
                .map(|icon| icon.working_directory().unwrap_or_default()),
        )),
        Max(width(
            FILE,
            icons.iter().map(|icon| icon.file().unwrap_or_default()),
        )),
        Max(width(
            COMMENT,
            icons.iter().map(|icon| icon.comment().unwrap_or_default()),
        )),
        Max(width(
            APP_USER_MODEL_ID,
            icons
                .iter()
                .map(|icon| icon.app_user_model_id().unwrap_or_default()),
        )),
        Max(width(
            APP_USER_MODEL_TOAST_ACTIVATOR_CLSID,
            icons.iter().map(Icon::app_user_model_toast_activator_clsid),
        )),
        Max(width(INDEX, icons.iter().map(Icon::index))),
        Max(width(SHOW_COMMAND, icons.iter().map(Icon::show_command))),
        Max(width(
            CLOSE_ON_EXIT,
            icons.iter().map(|icon| icon.close_on_exit().as_str()),
        )),
        Max(width(HOTKEY, icons.iter().map(Icon::hotkey))),
        Max(width(FLAGS, icons.iter().map(Icon::flags))),
    ]
}
