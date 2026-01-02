#[cfg(feature = "crossterm")]
pub use self::crossterm::CrosstermDriver;

#[cfg(feature = "crossterm")]
mod crossterm;
