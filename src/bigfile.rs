//! Module for loading and manipulating Relic .big files.

use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Read,
    path::Path,
};

use bytemuck::{Pod, Zeroable};
use derive_more::From;

use crate::Result;

/// Header bytes found in the beginning every .big file
const BF_FILE_HEADER: &[u8; 7] = b"RBF1.23";

/// Represents a Relic .big file handle.
pub(crate) struct BigFile {
    /// The .big file handle.
    handle: File,
    /// The 'table of contents' section of the .big file.
    toc: BigTOC,
}

/// Represents the 'table of contents' section of a .big file.
pub(crate) struct BigTOC {
    /// The number of files contained in the .big file.
    num_files: i32,
    /// Flags of a .big file.
    /// Currently only the lowest bit is used as a flag,
    /// 1 if the .big file is sorted, 0 if the .big file is unsorted
    flags: i32,
    /// A list of all the file entries (with location and size) in the .big file.
    file_entries: Vec<BigTOCFileEntry>,
}

/// Represents a file entry in the 'table of contents' section of a .big file.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
struct BigTOCFileEntry {
    /// A 32-bit CRC created from the first half of the file name.
    crc1: u32,
    /// A 32-bit CRC created from the second half of the file name.
    crc2: u32,
    /// The length of the file name.
    name_length: u16,
    /// An unused padding field for correct parsing.
    _padding1: u16,
    /// The amount of space this file takes up in the .big file.
    stored_length: u32,
    /// The actual size of this file.
    real_length: u32,
    /// The location of this file in the .big file.
    offset: u32,
    /// UNIX timestamp of the file.
    /// It can only represent dates up to January 19, 2038,
    /// 03:14:07 UTC because the timestamp is only 32 bits.
    timestamp: i32,
    /// A flag indicating whether the file is compressed or not.
    compression_type: u8,
    /// An unused padding field for correct parsing.
    _padding2: [u8; 3],
}

pub(crate) fn add(
    bigfile_name: &str,
    file_names: &[&str],
    compress: bool,
    is_newer: bool,
    store_paths: bool,
    console_output: bool,
) {
}

/// Opens a .big file.
pub(crate) fn open(file_path: &Path) -> Result<BigFile> {
    let mut big_file = File::open(file_path)?;

    verify_header(&mut big_file)?;

    let toc = read_toc(&mut big_file)?;

    Ok(BigFile {
        handle: big_file,
        toc,
    })
}

/// Verifies the header of the .big file.
fn verify_header(file: &mut File) -> Result<()> {
    let mut header_bytes: [u8; 7] = [0; 7];
    file.read_exact(&mut header_bytes)?;
    if header_bytes == *BF_FILE_HEADER {
        Ok(())
    } else {
        Err(crate::Error::BigFile(Error::InvalidHeader {
            actual: header_bytes,
            expected: *BF_FILE_HEADER,
        }))
    }
}

/// Reads the 'table of contents' section of the .big file.
fn read_toc(file: &mut File) -> Result<BigTOC> {
    let mut num_files: [u8; 4] = [0; 4];
    file.read_exact(&mut num_files)?;
    let mut flags: [u8; 4] = [0; 4];
    file.read_exact(&mut flags)?;
    let num_files: i32 = bytemuck::try_cast(num_files)?;
    let flags: i32 = bytemuck::try_cast(flags)?;
    let file_entries: Vec<BigTOCFileEntry> = if num_files > 0 {
        let mut file_entries = vec![
            BigTOCFileEntry::default();
            num_files
                .try_into()
                .map_err(|_err| crate::Error::ConversionError)?
        ];
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

/// An enum representing all errors internal to the bigfile module.
#[derive(Debug, From)]
pub enum Error {
    /// The header of opened .big file does not match the expected header bytes.
    #[from]
    InvalidHeader {
        /// The header of the opened .big file.
        actual: [u8; 7],
        /// The expected header bytes.
        expected: [u8; 7],
    },
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::bigfile;

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[test]
    fn should_open() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("res/test/Update.big");
        let big_file = bigfile::open(&path)?;

        let expected = 42_i32;
        if big_file.toc.num_files != expected {
            return Err(format!(
                "The number of files does not match: {}, {}",
                big_file.toc.num_files, expected
            )
            .into());
        }
        Ok(())
        // todo add more assertions
    }
}
