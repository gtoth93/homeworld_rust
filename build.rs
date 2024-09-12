//! A script for building the Homeworld crate.

use std::env;

use anyhow::Result;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=res");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["res/"];
    fs_extra::copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
