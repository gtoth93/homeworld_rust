use crate::bigfile;
use std::path::PathBuf;

#[test]
fn should_open() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("res/test/Update.big");
    let toc = bigfile::open(&path).expect("Could not read file");

    assert_eq!(toc.num_files, 42);
    // todo add more assertions
}
