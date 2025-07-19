/// Checks that get_dir returns a subdirectory and its entries can be listed.
#[test]
fn test_get_dir() {
    let dir = test_dir();
    let subdir = dir.get_dir("subdir");
    assert!(subdir.is_some());
    let subdir = subdir.unwrap();
    let entries = subdir.entries();
    let names: Vec<_> = entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    assert!(names.contains(&"gamma.txt".to_string()));
    assert!(names.contains(&"delta.txt".to_string()));
}

/// Checks that get_dir returns None for a non-existent subdirectory.
#[test]
fn test_get_dir_not_found() {
    let dir = test_dir();
    assert!(dir.get_dir("not_a_dir").is_none());
}

/// Checks that DirSet::get_dir returns the highest-precedence subdirectory.
#[test]
fn test_dirset_get_dir_override() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let subdir = set.get_dir("subdir");
    assert!(subdir.is_some());
    let subdir = subdir.unwrap();
    let entries = subdir.entries();
    let names: Vec<_> = entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    // Should contain gamma.txt and delta.txt from the base dir
    assert!(names.contains(&"gamma.txt".to_string()));
    assert!(names.contains(&"delta.txt".to_string()));
}

/// Checks that DirSet::get_dir returns None for a non-existent subdirectory.
#[test]
fn test_dirset_get_dir_not_found() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    assert!(set.get_dir("not_a_dir").is_none());
}

use fs_embed::*;

fn test_dir() -> Dir {
    Dir::from_str("tests/data")
}

fn test_override_dir() -> Dir {
    Dir::from_str("tests/data/override")
}

/// Checks that directory entries include expected files and subdirectories.
#[test]
fn test_dir_entries() {
    let dir = test_dir();
    let entries = dir.entries();
    let names: Vec<_> = entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    assert!(names.contains(&"alpha.txt".to_string()));
    assert!(names.contains(&"beta.txt".to_string()));
    assert!(names.contains(&"subdir".to_string()));
}

/// Checks that a file can be retrieved and its contents read correctly.
#[test]
fn test_get_file() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt");
    assert!(file.is_some());
    let file = file.unwrap();
    assert_eq!(file.file_name(), Some("alpha.txt"));
    let content = file.read_str().unwrap();
    assert_eq!(content.trim(), "Hello from alpha!");
}

/// Checks that getting a non-existent file returns None.
#[test]
fn test_get_file_not_found() {
    let dir = test_dir();
    assert!(dir.get_file("notfound.txt").is_none());
}

/// Checks that walk() finds all files in the directory tree.
#[test]
fn test_walk_flat() {
    let dir = test_dir();
    let files: Vec<_> = dir.walk().collect();
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap()).collect();
    assert!(names.contains(&"alpha.txt"));
    assert!(names.contains(&"beta.txt"));
    assert!(names.contains(&"gamma.txt"));
    assert!(names.contains(&"delta.txt"));
}

/// Checks that walk_override() yields overridden and new files as expected.
#[test]
fn test_walk_override() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let files: Vec<_> = set.walk_override().collect();
    let mut found_alpha = false;
    let mut found_epsilon = false;
    for f in files {
        if f.file_name() == Some("alpha.txt") {
            let content = f.read_str().unwrap();
            assert_eq!(content.trim(), "Overridden alpha!");
            found_alpha = true;
        }
        if f.file_name() == Some("epsilon.txt") {
            found_epsilon = true;
        }
    }
    assert!(found_alpha);
    assert!(found_epsilon);
}

/// Checks that get_file returns the overridden file from the higher-precedence root.
#[test]
fn test_dirset_get_file_override() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let file = set.get_file("alpha.txt").unwrap();
    let content = file.read_str().unwrap();
    assert_eq!(content.trim(), "Overridden alpha!");
}

/// Checks that get_file returns a non-overridden file from the lower-precedence root.
#[test]
fn test_dirset_get_file_non_override() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let file = set.get_file("beta.txt").unwrap();
    let content = file.read_str().unwrap();
    assert_eq!(content.trim(), "Beta file content");
}

/// Checks that entries() returns all immediate entries from all roots.
#[test]
fn test_dirset_entries() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let entries = set.entries();
    let names: Vec<_> = entries.iter().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect();
    assert!(names.contains(&"alpha.txt".to_string()));
    assert!(names.contains(&"beta.txt".to_string()));
    assert!(names.contains(&"subdir".to_string()));
    assert!(names.contains(&"epsilon.txt".to_string()));
}

/// Checks that file metadata (size, etc.) is accessible and valid.
#[test]
fn test_file_metadata() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    let meta = file.metadata().unwrap();
    assert!(meta.size > 0);
}

/// Checks that file extension is correctly returned.
#[test]
fn test_file_extension() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    assert_eq!(file.extension(), Some("txt"));
}

/// Checks that absolute_path() returns an absolute path for a file.
#[test]
fn test_file_absolute_path() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    let abs = file.absolute_path();
    assert!(abs.is_absolute());
}

/// Checks that is_embedded() returns false for a directory from the filesystem.
#[test]
fn test_dir_is_embedded_false() {
    let dir = test_dir();
    assert!(!dir.is_embedded());
}

/// Checks that is_embedded() returns false for a file from the filesystem.
#[test]
fn test_file_is_embedded_false() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    assert!(!file.is_embedded());
}

/// Checks that DirEntry correctly identifies files and directories.
#[test]
fn test_direntry_is_file_and_dir() {
    let dir = test_dir();
    let entries = dir.entries();
    let mut found_file = false;
    let mut found_dir = false;
    for e in entries {
        if e.is_file() { found_file = true; }
        if e.is_dir() { found_dir = true; }
    }
    assert!(found_file);
    assert!(found_dir);
}

/// Checks that DirEntry can be converted into File or Dir as appropriate.
#[test]
fn test_direntry_into_file_and_dir() {
    let dir = test_dir();
    for entry in dir.entries() {
        if entry.is_file() {
            assert!(entry.clone().into_file().is_some());
        }
        if entry.is_dir() {
            assert!(entry.clone().into_dir().is_some());
        }
    }
}

/// Checks that path() and absolute_path() on DirEntry do not panic.
#[test]
fn test_direntry_path_and_absolute_path() {
    let dir = test_dir();
    for entry in dir.entries() {
        let _ = entry.path();
        let _ = entry.absolute_path();
    }
}

/// Checks that walk() on DirSet finds all files from all roots.
#[test]
fn test_dirset_walk_all_files() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let files: Vec<_> = set.walk().collect();
    let mut found_alpha = false;
    let mut found_beta = false;
    let mut found_epsilon = false;
    for f in files {
        if f.file_name() == Some("alpha.txt") { found_alpha = true; }
        if f.file_name() == Some("beta.txt") { found_beta = true; }
        if f.file_name() == Some("epsilon.txt") { found_epsilon = true; }
    }
    assert!(found_alpha);
    assert!(found_beta);
    assert!(found_epsilon);
}

/// Checks that walk_override() yields unique files by relative path.
#[test]
fn test_dirset_walk_override_unique() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let files: Vec<_> = set.walk_override().collect();
    let mut seen = std::collections::HashSet::new();
    for f in files {
        let path = f.path().to_owned();
        assert!(seen.insert(path));
    }
}

/// Checks that walk() yields at least as many files as walk_override().
#[test]
fn test_dirset_walk_vs_walk_override() {
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let all: Vec<_> = set.walk().map(|f| f.path().to_owned()).collect();
    let unique: Vec<_> = set.walk_override().map(|f| f.path().to_owned()).collect();
    assert!(all.len() >= unique.len());
}

/// Checks that DirSet behaves correctly when empty.
#[test]
fn test_dirset_empty() {
    let set = DirSet::new(vec![]);
    assert_eq!(set.entries().len(), 0);
    assert!(set.get_file("alpha.txt").is_none());
    assert_eq!(set.walk().count(), 0);
    assert_eq!(set.walk_override().count(), 0);
}


/// Checks that override order in DirSet affects which file is returned.
#[test]
fn test_dirset_ordering_override() {
    let set1 = DirSet::new(vec![test_override_dir(), test_dir()]);
    let set2 = DirSet::new(vec![test_dir(), test_override_dir()]);
    let file1 = set1.get_file("alpha.txt").unwrap().read_str().unwrap();
    let file2 = set2.get_file("alpha.txt").unwrap().read_str().unwrap();
    assert_ne!(file1, file2);
}

/// Checks that File implements PartialEq correctly for same file.
#[test]
fn test_file_partial_eq() {
    let dir = test_dir();
    let f1 = dir.get_file("alpha.txt").unwrap();
    let f2 = dir.get_file("alpha.txt").unwrap();
    assert_eq!(f1, f2);
}

/// Checks that Dir implements PartialEq correctly for same directory.
#[test]
fn test_dir_partial_eq() {
    let d1 = test_dir();
    let d2 = test_dir();
    assert_eq!(d1, d2);
}

/// Checks that DirEntry implements PartialEq for entries with same path.
#[test]
fn test_direntry_partial_eq() {
    let dir = test_dir();
    let entries = dir.entries();
    for e1 in &entries {
        for e2 in &entries {
            if e1.path() == e2.path() {
                assert_eq!(e1, e2);
            }
        }
    }
}

/// Checks that hashing DirSet's walk_override is consistent across calls.
#[test]
fn test_dirset_hash_consistency() {
    use std::collections::HashSet;
    let set = DirSet::new(vec![test_dir(), test_override_dir()]);
    let files: HashSet<_> = set.walk_override().collect();
    let files2: HashSet<_> = set.walk_override().collect();
    for f in files.iter() {
        println!("File: {:?}", f.path());
    }

    println!("---------------------------------------------------");

    for f in files2.iter() {
        println!("File2: {:?}", f.path());
    }

    println!("---------------------------------------------------");
    println!("Files: {:?}", files.len());
    println!("Files2: {:?}", files2.len());
    assert_eq!(files, files2);
}

/// Checks that file contents can be read as bytes.
#[test]
fn test_file_read_bytes() {
    let dir = test_dir();
    let file = dir.get_file("alpha.txt").unwrap();
    let bytes = file.read_bytes().unwrap();
    assert!(!bytes.is_empty());
}

/// Checks that reading a file with invalid UTF-8 returns an error.
#[test]
fn test_file_read_str_invalid_utf8() {
    use std::io::Write;
    use std::fs;
    use std::path::Path;
    // Create a unique temp directory for this test
    let temp_dir = tempfile::Builder::new()
        .prefix("fs_embed_test_bad_utf8_")
        .tempdir()
        .expect("create temp dir");
    let file_path = temp_dir.path().join("bad_utf8.bin");
    let mut f = fs::File::create(&file_path).unwrap();
    f.write_all(&[0xff, 0xfe, 0xfd]).unwrap();
    // Use Dir::from_path to point to the temp dir
    let dir = Dir::from_path(temp_dir.path());
    let file = dir.get_file("bad_utf8.bin").unwrap();
    assert!(file.read_str().is_err());
    // temp_dir is deleted automatically
}

/// Checks that is_embedded() is false for all DirEntry from filesystem.
#[test]
fn test_direntry_is_embedded_false() {
    let dir = test_dir();
    for entry in dir.entries() {
        assert!(!entry.is_embedded());
    }
}

/// Checks that DirEntry can be cloned and hash/eq are consistent.
#[test]
fn test_direntry_clone_hash_eq() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let dir = test_dir();
    for entry in dir.entries() {
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        entry.hash(&mut h1);
        entry.clone().hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
        assert_eq!(entry, entry.clone());
    }
}
