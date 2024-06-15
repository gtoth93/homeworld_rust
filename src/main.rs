//! This crate is the game Homeworld from 1999, rewritten from the ground up in Rust.

pub use self::error::{Error, Result};

mod bigfile;
mod error;

/// The entry point for the game
fn main() -> Result<()> {
    // TODO check if another instance is already running

    // TODO load options file if it exists

    // TODO load keyboard redefinitions

    // TODO process command line

    // TODO preinitialize game systems

    // TODO create window (with debug window if enabled)

    // TODO initialize game systems

    // TODO start main loop

    // TODO handle logging based on Debug or Release profile

    Ok(())
}
