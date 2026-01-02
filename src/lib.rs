use std::io;

use crossterm::{
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use vtui_core::{component::Component, runtime::{EventSource, Runtime}};

pub use vtui_core::events;
pub use vtui_core::runtime;

pub mod prelude {
    pub use vtui_core::component::Component;
    pub use vtui_macros::component;
}

/// Starts a [`Runtime`] built using the `factory` function.
///
/// The factory is invoked synchronously to register components, draw callbacks, listeners, and
/// side effects. Once the runtime is built, registration is closed and execution begins.
pub fn launch(factory: fn(&mut Component)) -> anyhow::Result<()> {
    let mut root = Component::default();

    factory(&mut root);

    let source = EventSource::default();
    let mut runtime = Runtime::new(root);

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableFocusChange,
        EnableMouseCapture,
    )?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // The main event loop which presents UI frames and consumes runtime events.
    loop {
        terminal.draw(|f| {
            runtime.draw(f);
        })?;

        runtime.update(source.recv().unwrap());

        if runtime.should_exit() {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture,
    )?;

    Ok(())
}
