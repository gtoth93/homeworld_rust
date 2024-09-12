//! Module for loading and manipulating Relic .big files.

use derive_more::From;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::{
    fmt::{Display, Formatter},
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Header bytes found at the beginning of every .big file
const BF_FILE_HEADER: &[u8; 7] = b"RBF1.23";

/// Represents a Relic .big file handle.
pub(crate) struct BigFile {
    /// The .big file handle.
    handle: File,
    /// The 'table of contents' section of the .big file.
    toc: BigTOC,
}

/// Represents the 'table of contents' section of a .big file.
#[derive(Default)]
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
#[derive(AsBytes, Clone, Default, FromBytes, FromZeroes)]
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

/// Adds files to a .big file.
#[allow(clippy::print_stdout)]
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) fn add(
    bigfile_path: &Path,
    file_names: &[&str],
    compress: bool,
    is_newer: bool,
    move_files: bool,
    store_paths: bool,
    console_output: bool,
) -> Result<()> {
    let mut exe_dir = std::env::current_exe()?;
    exe_dir.pop();

    let mut big_file = if bigfile_path.try_exists()? {
        let big_file = open(bigfile_path)?;
        if console_output {
            println!("Updating {bigfile_path:?}");
        }
        big_file
    } else {
        if console_output {
            println!("Creating {}", bigfile_path.display());
        }
        create_new(bigfile_path)?
    };

    let temp_dir = tempfile::tempdir_in(exe_dir)?;
    let mut temp_file = tempfile::tempfile_in(temp_dir.path())?;

    std::io::copy(&mut big_file.handle, &mut temp_file)?;

    for &file_name in file_names {
        let mut filelist_lines = None;
        if file_name.starts_with('@') {
            let mut chars = file_name.chars();
            chars.next();
            let file_path = Path::new(chars.as_str()).canonicalize()?;
            let file = File::open(&file_path)?;
            filelist_lines = Some(BufReader::new(file).lines());
            if move_files && console_output {
                println!("Warning: Cannot (re)move files from filelist: {file_path:?}");
            }
        }
        loop {
            let file_name = if let Some(ref mut filelist_lines) = filelist_lines {
                match filelist_lines.next() {
                    Some(line) => line?,
                    None => break,
                }
            } else {
                String::from(file_name)
            };
        }
    }

    Ok(())
    // create temp file
    // if unsuccessful, print error to console, return 0
    // if big file exists at the path
    //      open big file in read mode
    //      if unsuccessful, print error to console, return 0
    //      verify big file header
    //      if unsuccessful, print error to console, return 0
    //      close file
    // else create new big file
    // copy big file to temp file
    // if unsuccessful, print error to console, return 0
    // create a vector of file handles for files to be moved
    // for each file to be added
    //      if first character of file path is @
    //          treat the file path as a file list
    //          ignore the @ in the file list
    //          fix all slashes in file path
    //          open file list in read mode
    //          if unsuccessful, print error to console, remove temp file, free the vector, return 0
    //          if move_files is set, then print a warning that files in a file list cannot be moved
    //      do
    //          if file path is a file list
    //              if end of file list is reached then break
    //              get a line from file list as a file name
    //              if line cannot be read as a string or line is empty, then continue
    //
    //          fix slashes in file path
    //          if storing full paths is not set, then extract the file name from path
    //          else use the full path
    //          print out the file path
    //          add file to big file
    //          if unsuccessful, remove the temp file, free the vector, return 0
    //          else
    //              if file was added, then print the result
    //                  if move files was set and file is not a file list, then mark the file for deletion
    //              else if file was updated, then print the result
    //                  if move files was set and file is not a file list, then mark the file for deletion
    //              else if file was unchanged, then print the result, do not mark the file for deletion
    //              else
    //                  do not mark the file for deletion
    //              if stored file name and file path are different, then print out the file name
    //      while processing a file list
    //      if we processed the file list, then close file list
    // for each file marked for deletion
    //      delete the file
    //      if deletion is unsuccessful, print an error message
    // if at least one file was added
    //      print a message announcing sort operation
    //      sort table of contents on temp file
    //      if unsuccessful, print an error message, return 0
    // if files added + files updated = 0, then print nothing to do
    // else
    //      if files were added, then print out how many
    //      if files were updated, then print out how many
    // replace the original file with the temp file
    // if unsuccessful, print an error message, return 0
    // delete the temp file
    // if unsuccessful, print an error message, return 0
    // return 1
}

pub(crate) fn open(file_path: &Path) -> Result<BigFile> {
    open_with_options(file_path, OpenOptions::new().read(true))
}

/// Opens a .big file using the supplied options.
pub(crate) fn open_with_options(file_path: &Path, options: &OpenOptions) -> Result<BigFile> {
    let mut handle = options.open(file_path)?;

    verify_header(&mut handle, file_path)?;

    let toc = read_toc(&mut handle)?;

    let big_file = BigFile { handle, toc };
    Ok(big_file)
}

/// Verifies the header of the .big file.
fn verify_header(file: &mut File, file_path: &Path) -> Result<()> {
    let mut header_bytes: [u8; 7] = [0; 7];
    file.read_exact(&mut header_bytes)?;
    if header_bytes == *BF_FILE_HEADER {
        Ok(())
    } else {
        let err = Error::InvalidHeader {
            file_path: file_path.to_path_buf(),
            actual: header_bytes,
            expected: *BF_FILE_HEADER,
        };
        Err(err)
    }
}

/// Reads the 'table of contents' section of the .big file.
fn read_toc(file: &mut File) -> Result<BigTOC> {
    let mut num_files: i32 = 0;
    // TODO file.read_exact(num_files.as_bytes_mut())?;
    file.read_exact(AsBytes::as_bytes_mut(&mut num_files))?;
    let mut flags: i32 = 0;
    // TODO file.read_exact(flags.as_bytes_mut())?;
    file.read_exact(AsBytes::as_bytes_mut(&mut flags))?;
    let file_entries: Vec<BigTOCFileEntry> = if num_files > 0 {
        let mut file_entries = vec![BigTOCFileEntry::default(); num_files.try_into()?];
        file.read_exact(file_entries.as_mut_slice().as_bytes_mut())?;
        file_entries
    } else {
        Vec::new()
    };
    let toc = BigTOC {
        num_files,
        flags,
        file_entries,
    };
    Ok(toc)
}

/// Creates a new .big file
fn create_new(file_path: &Path) -> Result<BigFile> {
    let mut handle = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;
    handle.write_all(BF_FILE_HEADER)?;
    let toc = BigTOC::default();
    write_toc(&mut handle, &toc)?;
    Ok(BigFile { handle, toc })
}

/// Writes the table of contents to the .big file.
/// This function assumes that we are the correct offset in the file (AKA right after the header).
fn write_toc(file: &mut File, toc: &BigTOC) -> Result<()> {
    file.write_all(AsBytes::as_bytes(&toc.num_files))?;
    file.write_all(AsBytes::as_bytes(&toc.flags))?;
    if toc.num_files > 0_i32 {
        file.write_all(toc.file_entries.as_slice().as_bytes())?;
    }
    Ok(())
}

/// A specialized Result type for .big file operations.
type Result<T> = std::result::Result<T, Error>;

/// An enum representing all errors internal to the bigfile module.
#[derive(Debug, From)]
pub enum Error {
    /// The header of the opened .big file does not match the expected header bytes.
    #[from]
    InvalidHeader {
        /// The path of the opened .big file.
        file_path: PathBuf,
        /// The header of the opened .big file.
        actual: [u8; 7],
        /// The expected header bytes.
        expected: [u8; 7],
    },
    /// An I/O operation error.
    /// It usually happens when failing to open a .big file or when reading/writing its contents.
    #[from]
    Io(std::io::Error),
    /// A generic integer conversion error.
    #[from]
    FailedToConvertInt(std::num::TryFromIntError),
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
            let err_string = format!(
                "The number of files does not match: {}, {}",
                big_file.toc.num_files, expected
            );
            return Err(err_string.into());
        }

        Ok(())
        // todo add more assertions
    }
}
