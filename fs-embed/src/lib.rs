use std::{collections::VecDeque, path::PathBuf};

pub use fs_embed_macros::fs_embed;

pub struct FileMetaData {
    /// The last modification time of the file.
    pub modified: std::time::SystemTime,
    /// The size of the file in bytes.
    pub size: u64,
}

#[derive(Debug, Clone)]
enum InnerFile {
    Embed(include_dir::File<'static>),
    Path {
        root: std::path::PathBuf,
        path: std::path::PathBuf,
    },
}

impl PartialEq for InnerFile {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for InnerFile {}

impl std::hash::Hash for InnerFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl InnerFile {
    #[inline(always)]
    fn absolute_path(&self) -> &std::path::Path {
        match self {
            InnerFile::Embed(file) => file.path(),
            InnerFile::Path { path, .. } => path.as_path(),
        }
    }

    #[inline(always)]
    fn is_embedded(&self) -> bool {
        matches!(self, InnerFile::Embed(_))
    }

    #[inline(always)]
    pub fn path(&self) -> &std::path::Path {
        match self {
            InnerFile::Embed(dir) => dir.path(),
            InnerFile::Path { root, path } => path.strip_prefix(root).unwrap_or(path),
        }
    }
}


#[derive(Debug, Clone)]
enum InnerDir {
    Embed(include_dir::Dir<'static>, &'static str),
    Path {
        root: std::path::PathBuf,
        path: std::path::PathBuf,
    },
}

impl PartialEq for InnerDir {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for InnerDir {}

impl std::hash::Hash for InnerDir {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl InnerDir {

    fn into_dynamic(self) -> Self {
        match &self {
            InnerDir::Embed(dir, path) => 
                Self::Path { root: PathBuf::from(path), path: PathBuf::from(path).join(dir.path()) },
            InnerDir::Path { .. } => self,
        }
    }

    #[inline(always)]
    fn is_embedded(&self) -> bool {
        matches!(self, InnerDir::Embed(..))
    }

    #[inline(always)]
    fn path(&self) -> &std::path::Path {
        match self {
            InnerDir::Embed(dir, _) => dir.path(),
            InnerDir::Path { root, path } => path.strip_prefix(root).unwrap_or(path),
        }
    }

    #[inline(always)]
    fn absolute_path(&self) -> &std::path::Path {
        match self {
            InnerDir::Embed(dir, _) => dir.path(),
            InnerDir::Path { path, .. } => path.as_path(),
        }
    }
}

#[derive(Debug, Clone)]
enum InnerEntry {
    File(InnerFile),
    Dir(InnerDir),
}

impl PartialEq for InnerEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InnerEntry::File(a), InnerEntry::File(b)) => a == b,
            (InnerEntry::Dir(a), InnerEntry::Dir(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for InnerEntry {}

impl std::hash::Hash for InnerEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            InnerEntry::File(file) => {
                0u8.hash(state); // Differentiate file from dir
                file.hash(state)
            }
            InnerEntry::Dir(dir) => {
                1u8.hash(state); // Differentiate dir from file
                dir.hash(state)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a directory, which may be embedded or from the filesystem.
/// Provides methods to enumerate and access files and subdirectories.
/// Represents a directory, which may be embedded or from the filesystem.
/// Provides methods to enumerate and access files and subdirectories.
pub struct Dir {
    inner: InnerDir,
}

impl Dir {
    /// Creates a directory from an embedded `include_dir::Dir` and its root path.
    /// Intended for use in tests and advanced scenarios.
    pub const fn from_embedded(dir: include_dir::Dir<'static>, path: &'static str) -> Self {
        Self {
            inner: InnerDir::Embed(dir, path),
        }
    }

    /// Creates a new directory from the given path, relative to the manifest directory at build time.
    /// The path can be any valid subdirectory or file path.
    pub fn from_path(path: &std::path::Path) -> Self {
        const BASE_DIR: &'static str = env!("CARGO_MANIFEST_DIR");
        let base_path = std::path::PathBuf::from(BASE_DIR);
        Self {
            inner: InnerDir::Path {
                root: base_path.join(path),
                path: base_path.join(path),
            }
        }
    }

    /// Converts an embedded directory to a dynamic (filesystem-backed) directory if possible.
    /// For embedded directories, this will create a Path variant using the embedded root path.
    pub fn into_dynamic(self) -> Self {
        Self {
            inner: self.inner.into_dynamic(),
        }
    }

    /// Automatically converts to a dynamic directory if in debug mode (cfg!(debug_assertions)).
    /// In release mode, returns self unchanged.
    pub fn auto_dynamic(self) -> Self {
        if cfg!(debug_assertions) {
            return self.into_dynamic();
        } else {
            return self;
        }
    }

    /// Creates a new root directory from the given string path, relative to the manifest directory.
    /// The path must be a string literal or static string.
    pub fn from_str(path: &'static str) -> Self {
        Self::from_path(std::path::Path::new(path))
    }

    /// Returns true if this directory is embedded in the binary.
    pub fn is_embedded(&self) -> bool {
        self.inner.is_embedded()
    }

    /// Returns the relative path of this directory.
    pub fn path(&self) -> &std::path::Path {
        self.inner.path()
    }

    /// Returns the absolute path of this directory.
    pub fn absolute_path(&self) -> &std::path::Path {
        self.inner.absolute_path()
    }

    /// Returns all immediate entries (files and subdirectories) in this directory.
    /// This is a low-level API; prefer using higher-level methods for most use cases.
    #[doc(hidden)]
    pub fn entries(&self) -> Vec<DirEntry> {
        match &self.inner {
            InnerDir::Embed(dir, root) => dir
                .files()
                .map(|file| DirEntry {
                    inner: InnerEntry::File(InnerFile::Embed(file.clone())),
                })
                .chain(dir.dirs().map(|subdir| DirEntry {
                    inner: InnerEntry::Dir(InnerDir::Embed(subdir.clone(), root)),
                }))
                .collect(),
            InnerDir::Path { root, path } => {
                let mut entries = Vec::new();
                if let Ok(entries_iter) = std::fs::read_dir(path) {
                    for entry in entries_iter.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            entries.push(DirEntry {
                                inner: InnerEntry::File(InnerFile::Path {
                                    root: root.clone(),
                                    path: entry_path,
                                }),
                            });
                        } else if entry_path.is_dir() {
                            entries.push(DirEntry {
                                inner: InnerEntry::Dir(InnerDir::Path {
                                    root: root.clone(),
                                    path: entry_path,
                                }),
                            });
                        }
                    }
                }
                entries
            }
        }
    }

    /// Returns the file with the given name if it exists in this directory.
    /// The name is relative to the directory root.
    pub fn get_file(&self, name: &str) -> Option<File> {
        match &self.inner {
            InnerDir::Embed(dir, _) => {
                dir.get_file(dir.path().join(name)).map(|file| File {
                    inner: InnerFile::Embed(file.clone()),
                })
            },
            InnerDir::Path { root, path } => {
                let new_path = path.join(name);
                if new_path.is_file() {
                    Some(File {
                        inner: InnerFile::Path {
                            root: root.clone(),
                            path: new_path,
                        },
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Recursively walks all files in this directory and its subdirectories.
    /// Returns an iterator over all files found.
    pub fn walk(&self) -> impl Iterator<Item = File> {
        let mut queue: VecDeque<DirEntry> = VecDeque::from_iter(self.entries().into_iter());
        std::iter::from_fn(move || {
            while let Some(entry) = queue.pop_front() {
                match entry.inner {
                    InnerEntry::File(file) => return Some(File { inner: file }),
                    InnerEntry::Dir(dir) => queue.extend(Dir { inner: dir }.entries()),
                }
            }
            None
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a file, which may be embedded or from the filesystem.
/// Provides methods to access file contents and metadata.
pub struct File {
    inner: InnerFile,
}

impl File {
    /// Returns the file name as a string slice, if available.
    pub fn file_name(&self) -> Option<&str> {
        self.path().file_name().and_then(|name| name.to_str())
    }

    /// Returns the file extension as a string slice, if available.
    pub fn extension(&self) -> Option<&str> {
        self.path().extension().and_then(|ext| ext.to_str())
    }

    /// Returns the absolute path of this file.
    pub fn absolute_path(&self) -> &std::path::Path {
        self.inner.absolute_path()
    }

    /// Returns true if this file is embedded in the binary.
    pub fn is_embedded(&self) -> bool {
        self.inner.is_embedded()
    }

    /// Returns the relative path of this file.
    pub fn path(&self) -> &std::path::Path {
        self.inner.path()
    }

    /// Reads the file contents as bytes.
    pub fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
        match &self.inner {
            InnerFile::Embed(file) => Ok(file.contents().to_vec()),
            InnerFile::Path { path, .. } => std::fs::read(path),
        }
    }

    /// Reads the file contents as a UTF-8 string.
    /// Returns an error if the contents are not valid UTF-8.
    pub fn read_str(&self) -> std::io::Result<String> {
        match &self.inner {
            InnerFile::Embed(file) => std::str::from_utf8(file.contents())
                .map(str::to_owned)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            InnerFile::Path { path, .. } => std::fs::read_to_string(path),
        }
    }

    /// Returns the metadata for this file, such as modification time and size.
    pub fn metadata(&self) -> std::io::Result<FileMetaData> {
        match &self.inner {
            InnerFile::Embed(file) => {
                if let Some(metadata) = file.metadata() {
                    Ok(FileMetaData {
                        modified: metadata.modified(),
                        size: file.contents().len() as u64,
                    })
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to get embedded file metadata",
                    ))
                }
            }
            InnerFile::Path { path, .. } => {
                let metadata = std::fs::metadata(path)?;
                Ok(FileMetaData {
                    modified: metadata.modified()?,
                    size: metadata.len(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a directory entry, which may be a file or a directory.
pub struct DirEntry {
    inner: InnerEntry,
}

impl DirEntry {
    /// Creates a directory entry from a file.
    pub fn from_file(file: File) -> Self {
        Self {
            inner: InnerEntry::File(file.inner),
        }
    }

    /// Creates a directory entry from a directory.
    pub fn from_dir(dir: Dir) -> Self {
        Self {
            inner: InnerEntry::Dir(dir.inner),
        }
    }

    /// Returns the relative path of this entry.
    pub fn path(&self) -> &std::path::Path {
        match &self.inner {
            InnerEntry::File(file) => file.path(),
            InnerEntry::Dir(dir) => dir.path(),
        }
    }

    /// Returns the absolute path of this entry.
    pub fn absolute_path(&self) -> &std::path::Path {
        match &self.inner {
            InnerEntry::File(file) => file.absolute_path(),
            InnerEntry::Dir(dir) => dir.absolute_path(),
        }
    }

    /// Returns true if this entry is embedded in the binary.
    pub fn is_embedded(&self) -> bool {
        matches!(&self.inner, InnerEntry::File(InnerFile::Embed(_)))
            || matches!(&self.inner, InnerEntry::Dir(InnerDir::Embed(..)))
    }

    /// Returns true if this entry is a file.
    pub const fn is_file(&self) -> bool {
        matches!(&self.inner, InnerEntry::File(_))
    }

    /// Returns true if this entry is a directory.
    pub const fn is_dir(&self) -> bool {
        matches!(&self.inner, InnerEntry::Dir(_))
    }

    /// Converts this entry into a file, if it is a file.
    pub fn into_file(self) -> Option<File> {
        if let InnerEntry::File(file) = self.inner {
            Some(File { inner: file })
        } else {
            None
        }
    }

    /// Converts this entry into a directory, if it is a directory.
    pub fn into_dir(self) -> Option<Dir> {
        if let InnerEntry::Dir(dir) = self.inner {
            Some(Dir { inner: dir })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a set of root directories, supporting overlay and override semantics.
/// Later directories in the set can override files from earlier ones with the same relative path.
pub struct DirSet {
    /// The list of root directories, in order of increasing precedence.
    pub dirs: Vec<Dir>,
}

impl DirSet {
    /// Creates a new DirSet from the given list of directories.
    /// The order of directories determines override precedence.
    pub fn new(dirs: Vec<Dir>) -> Self {
        Self { dirs }
    }

    /// Returns all immediate entries from all root directories.
    /// Entries from later roots do not override earlier ones in this list.
    #[doc(hidden)]
    pub fn entries(&self) -> Vec<DirEntry> {
        self.dirs.iter().flat_map(|dir| dir.entries()).collect()
    }

    /// Returns the file with the given name, searching roots in reverse order.
    /// Files in later roots override those in earlier roots if the relative path matches.
    pub fn get_file(&self, name: &str) -> Option<File> {
        for dir in self.dirs.iter().rev() {
            if let Some(file) = dir.get_file(name) {
                return Some(file);
            }
        }
        None
    }

    /// Recursively walks all files in all root directories.
    /// Files with the same relative path from different roots are all included.
    pub fn walk(&self) -> impl Iterator<Item = File> {
        let mut queue: VecDeque<DirEntry> = VecDeque::with_capacity(self.dirs.len() * 128); // Assuming an average of 128 entries per directory
        for dir in self.dirs.iter() {
            queue.push_back(DirEntry::from_dir(dir.clone()));
        }
        std::iter::from_fn(move || {
            while let Some(entry) = queue.pop_front() {
                match entry.inner {
                    InnerEntry::File(file) => return Some(File { inner: file }),
                    InnerEntry::Dir(dir) => {
                        for child in( Dir { inner: dir }).entries().into_iter().rev() {
                            queue.push_front(child);
                        }
                    },
                }
            }
            None
        })
    }

    /// Recursively walks all files, yielding only the highest-precedence file for each relative path.
    /// This implements the override behaviour: later roots take precedence over earlier ones.
    pub fn walk_override(&self) -> impl Iterator<Item = File> {
        let mut history = std::collections::HashSet::new();
        let mut queue: VecDeque<DirEntry> = VecDeque::with_capacity(self.dirs.len() * 128); // Assuming an average of 128 entries per directory
        for dir in self.dirs.iter() {
            queue.push_front(DirEntry::from_dir(dir.clone()));
        }
        std::iter::from_fn(move || {
            while let Some(entry) = queue.pop_front() {
                match entry.inner {
                    InnerEntry::File(file) => {                        
                        if  history.insert(file.path().to_owned()) {                           
                            return Some(File { inner: file })
                        }
                    },
                    InnerEntry::Dir(dir) => {
                        for child in( Dir { inner: dir }).entries().into_iter() {
                            queue.push_front(child);
                        }
                    },
                }
            }
            None
        })
    }
}
