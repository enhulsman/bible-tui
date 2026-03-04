# bible-tui

A terminal Bible reader with vim-like navigation, built with Rust and ratatui.

## Features

- **3 bundled translations** — KJV, WEB (with red-letter text), and Statenvertaling (Dutch)
- **Full-text search** — in-memory search index, min 2 characters, results across all books
- **Bookmarks** — save and recall verse positions
- **Vim-like navigation** — `j/k` scrolling, `g/G` jump, `:goto` command mode
- **Import support** — add translations from MyBible SQLite, Zefania XML, or JSON files
- **Fast startup** — translations bundled as postcard binary, deserialized instantly

## Installation

```
cargo install --path .
```

## Usage

```
bible                   # Launch with default translation (KJV)
bible --translation sv  # Launch with Statenvertaling
bible import file.xml   # Import a translation file
```

Navigate with `j`/`k` to scroll, `Space`/`Backspace` for next/previous chapter,
`Tab` to open the book/chapter navigation panel, `/` to search, and `:goto John 3:16`
to jump to a reference.

## Keybindings

### Reading

| Key                    | Action                  |
|------------------------|-------------------------|
| `j` / `k` / `↑` / `↓` | Scroll up/down          |
| `f` / `b` / PgDn/PgUp | Page scroll             |
| `g` / `G`              | Top / Bottom            |
| Space / Backspace      | Next / Previous chapter |
| `/`                    | Search                  |
| `n` / `N`              | Next / Prev result      |
| `:`                    | Command mode            |
| `B`                    | Bookmark current verse  |
| `?`                    | Toggle help overlay     |
| `q` / Ctrl+C           | Quit                    |

### Navigation Panel (Tab)

| Key                    | Action                  |
|------------------------|-------------------------|
| `j` / `k` / `↑` / `↓` | Move through list       |
| `h` / `l` / `←` / `→` | Switch books / chapters |
| Enter                  | Go to selection         |
| Esc / Tab              | Close panel             |

### Search (/)

| Key          | Action           |
|--------------|------------------|
| Type to search | Min 2 characters |
| `↑` / `↓`   | Select result    |
| Enter        | Go to result     |
| Esc          | Close search     |

## Commands

| Command          | Action                            |
|------------------|-----------------------------------|
| `:q`             | Quit                              |
| `:goto <ref>`    | Go to reference (e.g. `John 3:16`) |
| `:t <name>`      | Switch translation (`kjv`/`web`/`sv`) |

## Import Formats

**MyBible** (`.mybible`, `.sqlite`, `.sqlite3`) — SQLite databases from the MyBible app.

**Zefania XML** (`.xml`) — open Bible XML format with `<XMLBIBLE>` root element.

**JSON** (`.json`) — simple JSON schema with books, chapters, and verses arrays.

Import with:
```
bible import path/to/translation.xml
```

## Design Decisions

- **Bundled postcard binary** — translations are parsed at build time and serialized
  with postcard for near-instant deserialization at startup
- **Pre-wrapped text** — text is wrapped before rendering to work around
  ratatui's line-based scrolling (ratatui#2342), so scroll offset maps 1:1 to screen rows
- **In-memory search index** — full-text search built on load, no external index files

## Tech Stack

[ratatui](https://ratatui.rs) ·
[crossterm](https://docs.rs/crossterm) ·
[textwrap](https://docs.rs/textwrap) ·
[rusqlite](https://docs.rs/rusqlite) ·
[postcard](https://docs.rs/postcard) ·
[quick-xml](https://docs.rs/quick-xml)
