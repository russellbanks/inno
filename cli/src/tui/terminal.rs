use std::{io, mem::DropGuard, panic};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use ratatui::DefaultTerminal;

/// Sets up the terminal for the viewer: raw mode, the alternate screen, and mouse capture.
///
/// Dropping the returned guard puts the terminal back as it was, including while a panic
/// unwinds. A panic hook also turns mouse capture off, for a panic that aborts instead.
///
/// # Errors
///
/// Returns an error if the terminal cannot be set up. Anything already set up is undone.
pub fn init() -> io::Result<DropGuard<DefaultTerminal, impl FnOnce(DefaultTerminal)>> {
    // This also installs a panic hook that restores raw mode and the screen.
    let terminal = DropGuard::new(ratatui::try_init()?, |_terminal| restore());
    install_panic_hook();
    execute!(io::stdout(), EnableMouseCapture)?;
    Ok(terminal)
}

fn restore() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
}

/// Chains a panic hook that turns mouse capture off, which ratatui's own hook does not, so a
/// panic does not leave the terminal reporting mouse movement as input.
fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        previous(info);
    }));
}
