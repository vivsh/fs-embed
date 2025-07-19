
# fs-embed

[![crates.io](https://img.shields.io/crates/v/fs-embed.svg)](https://crates.io/crates/fs-embed)
[![docs.rs](https://docs.rs/fs-embed/badge.svg)](https://docs.rs/fs-embed)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/fs-embed)](../LICENSE)

Embed directories into your binary at compile time, or read from disk at runtime, with a unified and ergonomic API. Supports overlays, dynamic mode, and directory composition.

---

## Features

- Embed directories or files at compile time using a single macro: `fs_embed!()`
- Switch between embedded and disk-backed (dynamic) mode at runtime
- Compose multiple directories with overlays using `DirSet`
- Access files and subdirectories by path, list entries, and walk recursively
- Overlay/override support: later directories take precedence
- No build scripts, no config, no env vars
- Robust error handling and path normalization

---

## Example Usage

```rust
use fs_embed::fs_embed;

// Embed the "static" folder
static STATIC: fs_embed::Dir = fs_embed!("static");

fn main() {
    // Use disk in debug, embedded in release
    let dir = STATIC.auto_dynamic();
    if let Some(file) = dir.get_file("css/style.css") {
        let content = file.read_str().unwrap();
        println!("{content}");
    }
}
```

---

## Macro Usage and Options

The `fs_embed!` macro expects a literal relative path inside your crate, resolved via `CARGO_MANIFEST_DIR`.

```rust
static DIR: fs_embed::Dir = fs_embed!("assets");
```

By default, in debug mode, files are read from disk for hot-reload; in release mode, files are embedded in the binary.

Example with options:

```rust
static DIR: fs_embed::Dir = fs_embed!("assets");
```


## Directory API

### Dir

- `Dir::get_file(path)` — Get a file by relative path
- `Dir::get_dir(path)` — Get a subdirectory by relative path
- `Dir::entries()` — List all immediate entries (files and subdirectories)
- `Dir::walk()` — Recursively yield all files
- `Dir::is_embedded()` — Returns `true` if directory is embedded
- `Dir::into_dynamic()` — Always use disk (dynamic) mode
- `Dir::auto_dynamic()` — Use disk in debug, embedded in release

### File

- `File::file_name()` — Get the file name
- `File::extension()` — Get the file extension
- `File::read_bytes()` — Read file contents as bytes
- `File::read_str()` — Read file contents as UTF-8 string
- `File::metadata()` — Get file metadata (size, modified time)

### DirSet (Overlays)

You can compose multiple directories using `DirSet` to support overlays and override semantics. Later directories in the set override files from earlier ones with the same relative path.

```rust
use fs_embed::{fs_embed, DirSet};

static BASE: fs_embed::Dir = fs_embed!("base");
static THEME: fs_embed::Dir = fs_embed!("theme");

let set = DirSet::new(vec![BASE, THEME]);
if let Some(file) = set.get_file("index.html") {
    // File from THEME if it exists, otherwise BASE
}
```

- Overlay precedence is left-to-right
- Only the highest-precedence file for each path is returned by `walk_override()`

---

## Tests

This crate includes comprehensive tests for all public API. To run tests:

```sh
cargo test -p fs-embed
```

---

## License

This crate is dual-licensed under either the MIT or Apache-2.0 license, at your option.
See [LICENSE-MIT](../LICENSE-MIT) and [LICENSE-APACHE](../LICENSE-APACHE) for details.

---


---


## Related Crate: [`rust-silos`](https://crates.io/crates/rust-silos)

If you only need to embed files and access them by path (without directory traversal, overlays, or virtual filesystem features), consider using [`rust-silos`](https://crates.io/crates/rust-silos). It provides a minimal, highly efficient, and allocation-free API for file embedding, ideal for simple use cases where directory APIs are not required.

---

## Links

- [Documentation (docs.rs)](https://docs.rs/fs-embed)
- [Crate (crates.io)](https://crates.io/crates/fs-embed)
- [Repository (GitHub)](https://github.com/vivsh/fs-embed)
