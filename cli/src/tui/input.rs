use std::{io, time::Duration};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Position;

/// Rows scrolled per mouse wheel notch.
const WHEEL_SCROLL_ROWS: isize = 3;

/// Something the user asked the viewer to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Quit,
    NextTab,
    PreviousTab,
    /// Move the selection up a row.
    Up,
    /// Move the selection down a row.
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    /// Scroll the view by this many rows, leaving the selection alone.
    Scroll(isize),
    /// The left mouse button went down here.
    Press(Position),
    /// The mouse moved here with the left button held.
    Drag(Position),
    /// The left mouse button was released.
    Release,
    /// The mouse moved here with no button held.
    Hover(Position),
}

/// Blocks for the next event, then also takes every event already queued, and returns what they
/// ask for. A fast spin of the mouse wheel produces a burst of events, and handling the whole
/// burst before the next redraw keeps scrolling smooth.
pub fn read_batch() -> io::Result<Vec<Action>> {
    let mut actions = Vec::new();
    actions.extend(action(event::read()?));
    while event::poll(Duration::ZERO)? {
        actions.extend(action(event::read()?));
    }
    Ok(actions)
}

/// Returns the action `event` asks for, if any.
#[must_use]
pub fn action(event: Event) -> Option<Action> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => key_action(key),
        Event::Mouse(mouse) => mouse_action(mouse),
        _ => None,
    }
}

fn key_action(key: KeyEvent) -> Option<Action> {
    if key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('c' | 'C')) {
        return Some(Action::Quit);
    }
    Some(match key.code {
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => Action::Quit,
        KeyCode::Left | KeyCode::Char('a' | 'A') => Action::PreviousTab,
        KeyCode::Right | KeyCode::Char('d' | 'D') => Action::NextTab,
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        _ => return None,
    })
}

fn mouse_action(mouse: MouseEvent) -> Option<Action> {
    let position = Position::new(mouse.column, mouse.row);
    Some(match mouse.kind {
        MouseEventKind::ScrollDown => Action::Scroll(WHEEL_SCROLL_ROWS),
        MouseEventKind::ScrollUp => Action::Scroll(-WHEEL_SCROLL_ROWS),
        MouseEventKind::Down(MouseButton::Left) => Action::Press(position),
        MouseEventKind::Drag(MouseButton::Left) => Action::Drag(position),
        MouseEventKind::Up(MouseButton::Left) => Action::Release,
        MouseEventKind::Moved => Action::Hover(position),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };
    use ratatui::layout::Position;

    use super::{Action, action};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn mouse(kind: MouseEventKind) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 4,
            row: 7,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn keys() {
        for (code, expected) in [
            (KeyCode::Char('q'), Action::Quit),
            (KeyCode::Char('Q'), Action::Quit),
            (KeyCode::Esc, Action::Quit),
            (KeyCode::Left, Action::PreviousTab),
            (KeyCode::Char('a'), Action::PreviousTab),
            (KeyCode::Right, Action::NextTab),
            (KeyCode::Char('D'), Action::NextTab),
            (KeyCode::Up, Action::Up),
            (KeyCode::Down, Action::Down),
            (KeyCode::PageUp, Action::PageUp),
            (KeyCode::PageDown, Action::PageDown),
            (KeyCode::Home, Action::Home),
            (KeyCode::End, Action::End),
        ] {
            assert_eq!(action(key(code)), Some(expected), "{code:?}");
        }
        assert_eq!(action(key(KeyCode::Char('x'))), None);
    }

    #[test]
    fn ctrl_c_quits() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(action(event), Some(Action::Quit));
        assert_eq!(action(key(KeyCode::Char('c'))), None);
    }

    #[test]
    fn only_key_presses_count() {
        let mut release = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(action(Event::Key(release)), None);
    }

    #[test]
    fn mouse_events() {
        let at = Position::new(4, 7);
        for (kind, expected) in [
            (MouseEventKind::ScrollDown, Some(Action::Scroll(3))),
            (MouseEventKind::ScrollUp, Some(Action::Scroll(-3))),
            (
                MouseEventKind::Down(MouseButton::Left),
                Some(Action::Press(at)),
            ),
            (
                MouseEventKind::Drag(MouseButton::Left),
                Some(Action::Drag(at)),
            ),
            (MouseEventKind::Up(MouseButton::Left), Some(Action::Release)),
            (MouseEventKind::Moved, Some(Action::Hover(at))),
            (MouseEventKind::Down(MouseButton::Right), None),
        ] {
            assert_eq!(action(mouse(kind)), expected, "{kind:?}");
        }
        assert_eq!(action(Event::FocusGained), None);
    }
}
