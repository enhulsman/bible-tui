#!/usr/bin/env python3
"""Convert HSV_Bijbel.pdf to SimpleJSON format for bible-tui import.

Usage:
    nix-shell -p python3Packages.pymupdf --run 'python tools/convert_hsv.py HSV_Bijbel.pdf'

Output:
    ~/.local/share/bible-tui/translations/hsv.json

Then import with:
    bible import ~/.local/share/bible-tui/translations/hsv.json
"""

import json
import os
import re
import sys

BOOK_ORDER = [
    "Genesis", "Exodus", "Leviticus", "Numeri", "Deuteronomium",
    "Jozua", "Richteren", "Ruth", "Samuel 1", "Samuel 2",
    "Koningen 1", "Koningen 2", "Kronieken 1", "Kronieken 2",
    "Ezra", "Nehemia", "Esther", "Job", "Psalmen", "Spreuken",
    "Prediker", "Hooglied", "Jesaja", "Jeremia", "Klaagliederen",
    "Ezechi\u00ebl", "Dani\u00ebl", "Hosea", "Jo\u00ebl", "Amos", "Obadja", "Jona",
    "Micha", "Nahum", "Habakuk", "Zefanja", "Hagga\u00ef", "Zacharia", "Maleachi",
    "Matthe\u00fcs", "Markus", "Lukas", "Johannes", "Handelingen", "Romeinen",
    "Korinthe 1", "Korinthe 2", "Galaten", "Efeze", "Filippenzen",
    "Kolossenzen", "Thessalonicenzen 1", "Thessalonicenzen 2",
    "Timothe\u00fcs 1", "Timothe\u00fcs 2", "Titus", "Filemon", "Hebre\u00ebn",
    "Jakobus", "Petrus 1", "Petrus 2", "Johannes 1", "Johannes 2",
    "Johannes 3", "Judas", "Openbaring"
]

# Expected chapter counts per book (for validation)
EXPECTED_CHAPTERS = [
    50, 40, 27, 36, 34, 24, 21, 4, 31, 24, 22, 25, 29, 36,
    10, 13, 10, 42, 150, 31, 12, 8, 66, 52, 5, 48, 12, 14,
    3, 9, 1, 4, 7, 3, 3, 3, 2, 14, 4, 28, 16, 24, 21, 28,
    16, 16, 13, 6, 6, 4, 4, 5, 3, 6, 4, 3, 1, 13, 5, 5, 3,
    5, 1, 1, 1, 22
]

# Patterns to strip
ARTIFACT_RE = re.compile(r'^Pagina \d+$')
VERSION_RE = re.compile(r'^Herziene Statenvertaling Versie$')
BOOK_INDEX_RE = re.compile(r'^Boek \d+')

# Parse patterns
CHAPTER_RE = re.compile(r'^Hoofdstuk (\d+)$')
VERSE_RE = re.compile(r'^(\d+):(\d+)\s+(.*)')
# Continuation: a line that doesn't match verse/chapter patterns


def extract_text(pdf_path):
    """Extract all text from PDF pages."""
    import fitz
    doc = fitz.open(pdf_path)
    lines = []
    for page in doc:
        text = page.get_text()
        for line in text.split('\n'):
            line = line.strip()
            if not line:
                continue
            if ARTIFACT_RE.match(line):
                continue
            if VERSION_RE.match(line):
                continue
            if BOOK_INDEX_RE.match(line):
                continue
            lines.append(line)
    doc.close()
    return lines


def rejoin_hyphenated(lines):
    """Rejoin hyphenated line breaks: 'word-\\nrest' -> 'wordrest'."""
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # Check if line ends with a hyphen and next line starts lowercase
        while (i + 1 < len(lines) and line.endswith('-')
               and lines[i + 1] and lines[i + 1][0].islower()):
            line = line[:-1] + lines[i + 1]
            i += 1
        result.append(line)
        i += 1
    return result


def parse_books(lines):
    """Parse lines into books with chapters and verses."""
    books = []
    current_book_idx = 0
    current_chapter = 0
    current_verse_num = 0
    current_verse_text = ""
    chapters = []
    verses = []

    def flush_verse():
        nonlocal current_verse_text, current_verse_num
        if current_verse_num > 0 and current_verse_text:
            verses.append({
                "verse": current_verse_num,
                "text": current_verse_text.strip()
            })
            current_verse_text = ""

    def flush_chapter():
        nonlocal verses, current_chapter
        flush_verse()
        if current_chapter > 0 and verses:
            chapters.append({
                "chapter": current_chapter,
                "verses": list(verses)
            })
            verses = []

    def flush_book():
        nonlocal chapters, current_book_idx
        flush_chapter()
        if chapters:
            book_name = BOOK_ORDER[current_book_idx] if current_book_idx < len(BOOK_ORDER) else f"Book {current_book_idx + 1}"
            books.append({
                "name": book_name,
                "number": current_book_idx + 1,
                "chapters": list(chapters)
            })
            chapters = []

    for line in lines:
        ch_match = CHAPTER_RE.match(line)
        if ch_match:
            ch_num = int(ch_match.group(1))
            if ch_num == 1 and current_chapter > 0:
                # New book detected (chapter reset)
                flush_book()
                current_book_idx += 1
            flush_chapter()
            current_chapter = ch_num
            continue

        verse_match = VERSE_RE.match(line)
        if verse_match:
            flush_verse()
            # The chapter:verse pattern - we use the verse number
            current_verse_num = int(verse_match.group(2))
            current_verse_text = verse_match.group(3)
            continue

        # Continuation line
        if current_verse_num > 0:
            current_verse_text += " " + line

    # Flush remaining
    flush_book()

    return books


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <HSV_Bijbel.pdf>", file=sys.stderr)
        sys.exit(1)

    pdf_path = sys.argv[1]
    if not os.path.exists(pdf_path):
        print(f"File not found: {pdf_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Extracting text from {pdf_path}...")
    lines = extract_text(pdf_path)
    print(f"  {len(lines)} lines extracted")

    lines = rejoin_hyphenated(lines)
    print(f"  {len(lines)} lines after rejoin")

    print("Parsing books/chapters/verses...")
    books = parse_books(lines)

    # Validation
    total_verses = 0
    for i, book in enumerate(books):
        ch_count = len(book["chapters"])
        v_count = sum(len(ch["verses"]) for ch in book["chapters"])
        total_verses += v_count
        expected = EXPECTED_CHAPTERS[i] if i < len(EXPECTED_CHAPTERS) else "?"
        status = "OK" if ch_count == expected else f"WARN (expected {expected})"
        print(f"  {book['number']:>2}. {book['name']:<25} {ch_count:>3} chapters, {v_count:>4} verses  {status}")

    print(f"\nTotal: {len(books)} books, {total_verses} verses")

    if len(books) != 66:
        print(f"WARNING: Expected 66 books, got {len(books)}", file=sys.stderr)

    # Output
    output_dir = os.path.expanduser("~/.local/share/bible-tui/translations")
    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, "hsv.json")

    result = {
        "name": "Herziene Statenvertaling",
        "abbreviation": "HSV",
        "language": "nl",
        "books": books
    }

    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(result, f, ensure_ascii=False, indent=2)

    print(f"\nWritten to: {output_path}")
    print(f"Import with: bible import {output_path}")


if __name__ == "__main__":
    main()
