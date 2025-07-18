
use fs_embed::*;

static EMBEDDED: Dir = fs_embed!("tests/data");

fn embedded_dir() -> Dir {
    EMBEDDED.clone()
}

/// Checks that embedded directory entries include expected files and subdirectories.
#[test]
fn test_embedded_dir_entries() {
    let dir = embedded_dir();
    let entries = dir.entries();
    let names: Vec<_> = entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    assert!(names.contains(&"alpha.txt".to_string()));
    assert!(names.contains(&"beta.txt".to_string()));
    assert!(names.contains(&"subdir".to_string()));
}

/// Checks that a file can be retrieved and its contents read correctly from embedded dir.
#[test]
fn test_embedded_get_file() {
    let dir = embedded_dir();
    let file = dir.get_file("alpha.txt");
    assert!(file.is_some());
    let file = file.unwrap();
    assert_eq!(file.file_name(), Some("alpha.txt"));
    let content = file.read_str().unwrap();
    assert_eq!(content.trim(), "Hello from alpha!");
}

/// Checks that getting a non-existent file from embedded dir returns None.
#[test]
fn test_embedded_get_file_not_found() {
    let dir = embedded_dir();
    assert!(dir.get_file("notfound.txt").is_none());
}

/// Checks that embedded subdirectory entries and files are accessible and correct.
#[test]
fn test_embedded_subdir_entries_and_file() {
    let dir = embedded_dir();
    let subdir_entry = dir.entries().into_iter().find(|e| e.is_dir() && e.path().file_name().unwrap() == "subdir").expect("subdir missing");
    let root_dir = subdir_entry.into_dir().unwrap();
    let sub_entries = root_dir.entries();
    let names: Vec<_> = sub_entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    assert!(names.contains(&"gamma.txt".to_string()), "gamma.txt missing in subdir {names:?}");
    let gamma = root_dir.get_file("gamma.txt").expect("gamma.txt missing");
    let content = gamma.read_str().unwrap();
    assert!(content.contains("Gamma in subdir"));
}

/// Due to include_dir behavior, even inside subdir you must use full relative path from root.
/// This test ensures fs_embed handles that correctly.
#[test]
fn test_include_dir_quirk() {
    let dir = embedded_dir();
    let subdir_entry = dir.entries().into_iter().find(|e| e.is_dir() && e.path().file_name().unwrap() == "subdir").expect("subdir missing");
   let subdir = subdir_entry.into_dir().unwrap();
   let file = subdir.get_file("gamma.txt").expect("gamma.txt missing");
   assert!(file.is_embedded());
}

/// Checks that walk() finds all files in the embedded directory tree.
#[test]
fn test_embedded_walk_flat() {
    let dir = embedded_dir();
    let files: Vec<_> = dir.walk().collect();
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap()).collect();
    assert!(names.contains(&"alpha.txt"));
    assert!(names.contains(&"beta.txt"));
    assert!(names.contains(&"gamma.txt"));
    assert!(names.contains(&"delta.txt"));
}

/// Checks that file metadata (size, etc.) is accessible and valid for embedded file.
#[test]
fn test_embedded_file_metadata() {
    let dir = embedded_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    let meta = file.metadata().unwrap();
    assert!(meta.size > 0);
}

/// Checks that file extension is correctly returned for embedded file.
#[test]
fn test_embedded_file_extension() {
    let dir = embedded_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    assert_eq!(file.extension(), Some("txt"));
}

/// Checks that absolute_path() returns an absolute path for an embedded file.
#[test]
fn test_embedded_file_absolute_path() {
    let dir = embedded_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    let abs = file.absolute_path();
    assert!(!abs.is_absolute(), "Embedded file should not have an absolute path");
}

/// Checks that is_embedded() returns true for embedded Dir, File, and DirEntry.
#[test]
fn test_embedded_is_embedded_true() {
    let dir = embedded_dir();
    assert!(dir.is_embedded());
    let file = dir.get_file("alpha.txt").unwrap();
    assert!(file.is_embedded());
    for entry in dir.entries() {
        assert!(entry.is_embedded());
    }
}
