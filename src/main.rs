use std::path::Path;

use anyhow::Result;

mod bigfile;

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

    let main_toc = bigfile::open(&Path::new(env!("OUT_DIR")).join("res").join("Homeworld.big"))?;
    let update_toc = bigfile::open(&Path::new(env!("OUT_DIR")).join("res").join("Update.big"))?;
    Ok(())
}
