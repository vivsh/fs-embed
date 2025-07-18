/// Tests for the fs_embed! procedural macro.
use fs_embed::*;



/// Checks that fs_embed! returns a Dir that can read files in debug or release mode.
#[test]
fn test_fs_embed_basic() {
    let dir = fs_embed!("tests/data");
    let file = dir.get_file("alpha.txt").unwrap();
    let content = file.read_str().unwrap();
    assert!(content.contains("Hello from alpha!"));
}


/// Checks that fs_embed! returns None for missing files.
#[test]
fn test_fs_embed_missing_file() {
    let dir = fs_embed!("tests/data");
    assert!(dir.get_file("notfound.txt").is_none());
}
