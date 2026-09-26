# Autopilot: C1 whole-chapter ranges

- **Base branch:** `spec/c1-chapter-ranges`
- **Spec:** `spec/c1.md`
- **Check command:** `cargo test -p bible-core` (must exit 0)

## May edit

- `bible-core/src/**`

## Must not touch

- `bible-core/tests/**`
- `Cargo.toml`, `bible-core/Cargo.toml` (any `Cargo.toml`), `Cargo.lock`
- `spec/**`
- `build/**`
- any `build.rs` (including `bible-core/build.rs`)

Existing inline tests under `bible-core/src/**` are part of the check: do not
delete, weaken or `#[ignore]` them. The one base assertion that C1 reverses
has already been removed (spec DECISION D2).

## Promise

- `Rom 8-9` parses to `Rom 8:1 .. Rom 9:END_OF_CHAPTER`, and `Rom 8-8` equals
  `Rom 8`, for any book-name form, with `-` or en dash and optional spaces.
- Reversed (`Rom 9-8`), out-of-canon (`Rom 8-17`, `Oba 1-2`) and malformed
  chapter ranges return `None` without panicking.
- With only a context book, `8-9` is a chapter range in that book; with a
  context book and chapter, `8-9` stays a verse range as before.
- `Rom 8-9:5` parses to `Rom 8:1 .. Rom 9:5` (owner DECISION D1).
- Every form accepted before C1 (`Rom 8:5-9`, `John 3:16-18`, `Gen 1:1-2:3`)
  returns the same value; the public shape of `parse_range`, `VerseRef` and
  `VerseRange` is unchanged.
