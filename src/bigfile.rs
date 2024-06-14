use std::{fs::File, io::Read, path::Path};

use anyhow::{Error, Result};
use bytemuck::{Pod, Zeroable};

#[cfg(test)]
mod tests;

const BF_FILE_HEADER: &[u8; 7] = b"RBF1.23";

pub struct BigTOC {
    num_files: i32,
    flags: i32,
    file_entries: Vec<BigTOCFileEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct BigTOCFileEntry {
    crc1: u32,
    crc2: u32,
    name_length: u16,
    _padding1: u16,
    stored_length: u32,
    real_length: u32,
    offset: u32,
    timestamp: i32,
    compression_type: u8,
    _padding2: [u8; 3],
}

pub fn add(
    bigfile_name: &str,
    file_names: &[&str],
    compress: bool,
    is_newer: bool,
    store_paths: bool,
    console_output: bool,
) {
}

pub fn open(file_path: &Path) -> Result<BigTOC> {
    let mut big_file = File::open(file_path)?;

    verify_header(&mut big_file)?;

    let toc = read_toc(&mut big_file)?;

    Ok(toc)
}

fn verify_header(file: &mut File) -> Result<()> {
    let mut header_bytes: [u8; 7] = [0; 7];
    file.read_exact(&mut header_bytes)?;
    if header_bytes == *BF_FILE_HEADER {
        Ok(())
    } else {
        Err(Error::msg("Incorrect big file header"))
    }
}

fn read_toc(file: &mut File) -> Result<BigTOC> {
    let mut num_files: [u8; 4] = [0; 4];
    file.read_exact(&mut num_files)?;
    let mut flags: [u8; 4] = [0; 4];
    file.read_exact(&mut flags)?;
    let num_files: i32 = bytemuck::try_cast(num_files)?;
    let flags: i32 = bytemuck::try_cast(flags)?;
    let file_entries: Vec<BigTOCFileEntry> = if num_files > 0 {
        let mut file_entries = vec![BigTOCFileEntry::default(); num_files.try_into()?];
        file.read_exact(bytemuck::cast_slice_mut(file_entries.as_mut_slice()))?;
        file_entries
    } else {
        Vec::new()
    };
    Ok(BigTOC {
        num_files,
        flags,
        file_entries,
    })
}
