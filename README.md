# fs-embed

Embed directories into your binary at compile time with **zero runtime config** — supporting both **embedded** and **live (dynamic)** modes through a single API.

---

## 📝 Philosophy

`fs-embed` is built for **frictionless file access across environments**:

* ✅ **Always embedded**: `fs_embed!("dir")` embeds directories at compile time, even in debug builds.
* Internally, it uses **include_dir** to embed static directories.
* 🔁 **Switch to live mode** (e.g. for hot-reloading): Call `.into_dynamic()` or `.auto_dynamic()` — no need to change your file access logic.
* 🔗 **Overlay support**: Compose multiple embedded or dynamic directories using `DirSet`, with override precedence.
* 📩 **One macro, one API**: No cargo features, no env vars, no conditional compilation.

This enables:

* **Reproducible builds**: Assets are fully embedded in release binaries.
* **Fast iteration in development**: Read live files with a one-line change — no config, no rebuild.
* **Clean architecture**: Keep templates, config, and assets in sync with source and override them per environment.

---

## 🚀 Quick Start

```rust
use fs_embed::fs_embed;

// Embed the "static" folder — always at compile time, even in debug builds.
static STATIC: fs_embed::Dir = fs_embed!("static");

fn main() {
    // Use disk in debug builds, embedded in release
    let dir = STATIC.auto_dynamic();

    // Load a file (relative to the embedded root or disk path)
    if let Some(file) = dir.get_file("css/style.css") {
        let content = file.read_str().unwrap();
        println!("{content}");
    }
}
```

> 📌 `fs_embed!("static")` expects a **literal relative path** inside your crate, resolved via `CARGO_MANIFEST_DIR`.

---

## 🔄 Embedded vs Dynamic

Two methods let you switch modes at runtime:

* **`.into_dynamic()`**

  * Always reads from disk.
  * Useful for tests, hot-reload, or CLI tools.

  ```rust
  let dir = fs_embed!("templates").into_dynamic();
  ```

* **`.auto_dynamic()`**

  * Reads from disk in debug, embedded in release.
  * Best for development–production parity.

  ```rust
  let dir = fs_embed!("templates").auto_dynamic();
  ```

---

## 📚 Core API

| Method                             | Description                                               |
| ---------------------------------- | --------------------------------------------------------- |
| `Dir::get_file(path)`              | Get a file by relative path                               |
| `Dir::read_str()` / `read_bytes()` | Read file contents                                        |
| `Dir::walk()`                      | Recursively yield all files                               |
| `Dir::walk_override()`             | Recursively yield highest-precedence files (for overlays) |                              |
| `Dir::is_embedded()`               | Returns `true` if directory is embedded                   |
| `Dir::into_dynamic()`              | Convert to dynamic (disk-backed) mode                     |
| `Dir::auto_dynamic()`              | Use disk in debug, embedded in release                    |

---

## 🥃 Override Behavior

You can compose multiple directories using `DirSet`:

```rust
let dir = DirSet::new([
    fs_embed!("base"),
    fs_embed!("theme").auto_dynamic(),
]);

let file = dir.get_file("index.html"); // From theme if it exists, otherwise base
```

* Overlay precedence is left-to-right.
* Only the **first match** for each path is returned.
* `walk_override()` yields exactly one file per unique relative path.

---

## ⚠️ Notes

* `DirSet::walk()` traversal order is **not guaranteed** (OS-dependent).
* `Dir::entries()` preserves insertion order of embedded files and folders, but disk-backed order may vary.
* All embedded files are resolved at compile time. Dynamic mode reads directly from disk at runtime.

---

## 📦 Installation

```toml
[dependencies]
fs-embed = "0.1"
```

---

## 📖 Docs

Full API documentation: [docs.rs/fs-embed](https://docs.rs/fs-embed)

---
