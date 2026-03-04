# bible-tui

A terminal Bible reader with vim-like navigation, built with Rust and ratatui.

## Features

- **3 bundled translations** — KJV, WEB (with red-letter text), and Statenvertaling (Dutch)
- **Import translations** — add from MyBible SQLite, Zefania XML, or JSON files; loaded at runtime from SQLite library
- **Command autocomplete** — dropdown suggestions for commands, translations, and book names
- **Full-text search** — in-memory search index, min 2 characters, results across all books
- **Bookmarks** — save and recall verse positions
- **Vim-like navigation** — `j/k` scrolling, `g/G` jump, `:goto` command mode
- **State persistence** — remembers last position and translation across sessions
- **Fast startup** — bundled translations serialized as postcard binary, deserialized instantly

## Installation

```
cargo install --path .
```

## Usage

```
bible                        # Launch TUI (default: KJV, or last used translation)
bible import file.json       # Import a translation file into the library
```

### Importing Translations

Import adds a translation to the SQLite library (`~/.local/share/bible-tui/library.sqlite`). Imported translations appear alongside bundled ones in `:t` autocomplete.

```
bible import path/to/translation.xml
bible import path/to/translation.json
bible import path/to/translation.mybible
```

Supported formats:

- **MyBible** (`.mybible`, `.sqlite`, `.sqlite3`) — SQLite databases from the MyBible app
- **Zefania XML** (`.xml`) — open Bible XML format with `<XMLBIBLE>` root element
- **SimpleJSON** (`.json`) — JSON with `name`, `abbreviation`, `language`, and `books` array containing chapters and verses

### HSV (Herziene Statenvertaling)

A conversion tool is included for importing the HSV from a licensed PDF:

```
nix-shell -p python3Packages.pymupdf --run 'python tools/convert_hsv.py HSV_Bijbel.pdf'
bible import ~/.local/share/bible-tui/translations/hsv.json
```

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
| `g` / `G`              | First / Last item       |
| `f` / `b` / PgDn/PgUp | Page through chapters   |
| Enter                  | Go to selection         |
| Esc / Tab              | Close panel             |

### Search (/)

| Key            | Action           |
|----------------|------------------|
| Type to search | Min 2 characters |
| `↑` / `↓`     | Select result    |
| Enter          | Go to result     |
| Esc            | Close search     |

### Commands (:)

| Key            | Action                          |
|----------------|---------------------------------|
| Tab            | Accept autocomplete suggestion  |
| `↑` / `↓`     | Navigate suggestions            |
| Enter          | Execute command                 |
| Esc            | Close command mode              |

## Commands

| Command       | Action                                       |
|---------------|----------------------------------------------|
| `:q`          | Quit                                         |
| `:goto <ref>` | Go to reference (e.g. `John 3:16`, `Rev 22`) |
| `:t <name>`   | Switch translation (bundled or imported)      |

## Design Decisions

- **Bundled postcard binary** — translations are parsed at build time and serialized
  with postcard for near-instant deserialization at startup
- **SQLite library for imports** — imported translations stored in a single SQLite file;
  bulk-loaded into memory on switch with translation-specific book names
- **Pre-wrapped text** — text is wrapped before rendering to work around
  ratatui's line-based scrolling, so scroll offset maps 1:1 to screen rows
- **In-memory search index** — full-text search built on translation load, no external index files

## Tech Stack

[ratatui](https://ratatui.rs) ·
[crossterm](https://docs.rs/crossterm) ·
[textwrap](https://docs.rs/textwrap) ·
[rusqlite](https://docs.rs/rusqlite) ·
[postcard](https://docs.rs/postcard) ·
[quick-xml](https://docs.rs/quick-xml)
