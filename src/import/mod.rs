pub mod json;
pub mod mybible;
pub mod zefania;

use std::path::Path;

use crate::bible::canon::CANON;
use crate::bible::model::{
    BibleData, Book, Chapter, SectionHeading, TranslationId, TranslationInfo, Verse, VerseSpan,
};

#[derive(Debug)]
pub enum ImportFormat {
    MyBible,
    Zefania,
    SimpleJson,
}

#[derive(Debug)]
pub struct ImportedTranslation {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub language: String,
    pub books: Vec<ImportedBook>,
}

#[derive(Debug)]
pub struct ImportedBook {
    pub number: u8, // 1-66
    pub name: String,
    pub chapters: Vec<Chapter>,
}

/// Detect import format by file extension and content.
pub fn detect_format(path: &Path) -> Option<ImportFormat> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("mybible") | Some("sqlite") | Some("sqlite3") => Some(ImportFormat::MyBible),
        Some("xml") => {
            // Peek at content to distinguish Zefania from other XML
            if let Ok(content) = std::fs::read_to_string(path) {
                let start = &content[..content.len().min(500)];
                if start.contains("XMLBIBLE") || start.contains("Zefania") {
                    Some(ImportFormat::Zefania)
                } else {
                    None
                }
            } else {
                None
            }
        }
        Some("json") => Some(ImportFormat::SimpleJson),
        _ => None,
    }
}

/// Import a translation file into the library database.
pub fn import_file(
    path: &Path,
    db: &rusqlite::Connection,
) -> color_eyre::Result<String> {
    let format = detect_format(path)
        .ok_or_else(|| color_eyre::eyre::eyre!("Unknown file format: {}", path.display()))?;

    let translation = match format {
        ImportFormat::MyBible => mybible::parse(path)?,
        ImportFormat::Zefania => zefania::parse(path)?,
        ImportFormat::SimpleJson => json::parse(path)?,
    };

    let id = translation.id.clone();
    store_translation(db, &translation)?;
    Ok(id)
}

/// Initialize the library SQLite schema.
pub fn init_library(db: &rusqlite::Connection) -> color_eyre::Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS translations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            abbreviation TEXT NOT NULL,
            language TEXT NOT NULL,
            source_format TEXT
        );
        CREATE TABLE IF NOT EXISTS verses (
            translation TEXT NOT NULL,
            book INTEGER NOT NULL,
            chapter INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            text TEXT NOT NULL,
            is_red_letter BOOLEAN DEFAULT 0,
            PRIMARY KEY (translation, book, chapter, verse)
        );
        CREATE TABLE IF NOT EXISTS headings (
            translation TEXT NOT NULL,
            book INTEGER NOT NULL,
            chapter INTEGER NOT NULL,
            before_verse INTEGER NOT NULL,
            text TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS book_names (
            translation TEXT NOT NULL,
            book INTEGER NOT NULL,
            name TEXT NOT NULL,
            PRIMARY KEY (translation, book)
        );",
    )?;
    Ok(())
}

fn store_translation(
    db: &rusqlite::Connection,
    translation: &ImportedTranslation,
) -> color_eyre::Result<()> {
    // Remove old data if re-importing
    db.execute(
        "DELETE FROM verses WHERE translation = ?1",
        [&translation.id],
    )?;
    db.execute(
        "DELETE FROM headings WHERE translation = ?1",
        [&translation.id],
    )?;
    db.execute(
        "DELETE FROM translations WHERE id = ?1",
        [&translation.id],
    )?;
    db.execute(
        "DELETE FROM book_names WHERE translation = ?1",
        [&translation.id],
    )?;

    db.execute(
        "INSERT INTO translations (id, name, abbreviation, language, source_format)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            translation.id,
            translation.name,
            translation.abbreviation,
            translation.language,
            ""
        ],
    )?;

    let tx = db.unchecked_transaction()?;
    for book in &translation.books {
        tx.execute(
            "INSERT OR REPLACE INTO book_names (translation, book, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![translation.id, book.number, book.name],
        )?;
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                let text = verse.text();
                let is_red = verse.spans.iter().any(|s| matches!(s, VerseSpan::RedLetter(_)));
                tx.execute(
                    "INSERT OR REPLACE INTO verses (translation, book, chapter, verse, text, is_red_letter)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        translation.id,
                        book.number,
                        chapter.number,
                        verse.number,
                        text,
                        is_red
                    ],
                )?;
            }
            for heading in &chapter.headings {
                tx.execute(
                    "INSERT INTO headings (translation, book, chapter, before_verse, text)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        translation.id,
                        book.number,
                        chapter.number,
                        heading.before_verse,
                        heading.text
                    ],
                )?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// List all imported translations from the library database.
pub fn list_translations(db: &rusqlite::Connection) -> Vec<TranslationInfo> {
    let mut stmt = match db.prepare("SELECT id, name, abbreviation FROM translations") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let abbreviation: String = row.get(2)?;
        Ok(TranslationInfo {
            id: TranslationId::Imported(id),
            name,
            abbreviation,
        })
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    rows.filter_map(|r| r.ok()).collect()
}

/// Load an entire imported translation as BibleData from SQLite.
pub fn load_full_bible(db: &rusqlite::Connection, id: &str) -> Option<BibleData> {
    // Load book names for this translation
    let mut book_name_map = std::collections::HashMap::new();
    if let Ok(mut stmt) =
        db.prepare("SELECT book, name FROM book_names WHERE translation = ?1")
    {
        if let Ok(rows) = stmt.query_map(rusqlite::params![id], |row| {
            Ok((row.get::<_, u8>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                book_name_map.insert(row.0, row.1);
            }
        }
    }

    // Bulk load all verses
    let mut stmt = db
        .prepare(
            "SELECT book, chapter, verse, text, is_red_letter FROM verses \
             WHERE translation = ?1 ORDER BY book, chapter, verse",
        )
        .ok()?;

    let rows: Vec<(u8, u16, u8, String, bool)> = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return None;
    }

    // Bulk load all headings
    let mut heading_stmt = db
        .prepare(
            "SELECT book, chapter, before_verse, text FROM headings \
             WHERE translation = ?1 ORDER BY book, chapter, before_verse",
        )
        .ok()?;

    let heading_rows: Vec<(u8, u16, u8, String)> = heading_stmt
        .query_map(rusqlite::params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    // Group into books/chapters
    let mut books: Vec<Book> = Vec::new();
    let mut current_book_num: u8 = 0;
    let mut current_ch_num: u16 = 0;
    let mut current_verses: Vec<Verse> = Vec::new();
    let mut current_chapters: Vec<Chapter> = Vec::new();

    for (book_num, ch_num, verse_num, text, is_red) in &rows {
        // Skip invalid book numbers (deuterocanonical etc.)
        if *book_num == 0 || *book_num > 66 {
            continue;
        }

        if *book_num != current_book_num {
            // Finish previous chapter + book
            if !current_verses.is_empty() {
                let headings = collect_headings(&heading_rows, current_book_num, current_ch_num);
                current_chapters.push(Chapter {
                    number: current_ch_num,
                    verses: std::mem::take(&mut current_verses),
                    headings,
                });
            }
            if current_book_num != 0 && !current_chapters.is_empty() {
                let index = current_book_num - 1;
                let name = book_name_map
                    .get(&current_book_num)
                    .cloned()
                    .unwrap_or_else(|| CANON[index as usize].name.to_string());
                books.push(Book {
                    name,
                    code: CANON[index as usize].code.to_string(),
                    index,
                    chapters: std::mem::take(&mut current_chapters),
                });
            }
            current_book_num = *book_num;
            current_ch_num = *ch_num;
        } else if *ch_num != current_ch_num {
            // Finish previous chapter
            if !current_verses.is_empty() {
                let headings = collect_headings(&heading_rows, current_book_num, current_ch_num);
                current_chapters.push(Chapter {
                    number: current_ch_num,
                    verses: std::mem::take(&mut current_verses),
                    headings,
                });
            }
            current_ch_num = *ch_num;
        }

        let span = if *is_red {
            VerseSpan::RedLetter(text.clone())
        } else {
            VerseSpan::Plain(text.clone())
        };
        current_verses.push(Verse {
            number: *verse_num,
            spans: vec![span],
            paragraph_break: false,
        });
    }

    // Flush last chapter + book
    if !current_verses.is_empty() {
        let headings = collect_headings(&heading_rows, current_book_num, current_ch_num);
        current_chapters.push(Chapter {
            number: current_ch_num,
            verses: current_verses,
            headings,
        });
    }
    if current_book_num != 0 && !current_chapters.is_empty() {
        let index = current_book_num - 1;
        let name = book_name_map
            .get(&current_book_num)
            .cloned()
            .unwrap_or_else(|| CANON[index as usize].name.to_string());
        books.push(Book {
            name,
            code: CANON[index as usize].code.to_string(),
            index,
            chapters: current_chapters,
        });
    }

    if books.is_empty() {
        return None;
    }

    Some(BibleData {
        translation: TranslationId::Imported(id.to_string()),
        books,
    })
}

fn collect_headings(
    heading_rows: &[(u8, u16, u8, String)],
    book: u8,
    chapter: u16,
) -> Vec<SectionHeading> {
    heading_rows
        .iter()
        .filter(|(b, c, _, _)| *b == book && *c == chapter)
        .map(|(_, _, before_verse, text)| SectionHeading {
            before_verse: *before_verse,
            text: text.clone(),
        })
        .collect()
}

/// Load a chapter from the library database.
#[allow(dead_code)]
pub fn load_chapter(
    db: &rusqlite::Connection,
    translation_id: &str,
    book: u8,
    chapter: u16,
) -> Option<Chapter> {
    let mut stmt = db
        .prepare(
            "SELECT verse, text, is_red_letter FROM verses
             WHERE translation = ?1 AND book = ?2 AND chapter = ?3
             ORDER BY verse",
        )
        .ok()?;

    let verses: Vec<Verse> = stmt
        .query_map(rusqlite::params![translation_id, book, chapter], |row| {
            let number: u8 = row.get(0)?;
            let text: String = row.get(1)?;
            let is_red: bool = row.get(2)?;
            let span = if is_red {
                VerseSpan::RedLetter(text)
            } else {
                VerseSpan::Plain(text)
            };
            Ok(Verse {
                number,
                spans: vec![span],
                paragraph_break: false,
            })
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    if verses.is_empty() {
        return None;
    }

    // Load headings
    let mut heading_stmt = db
        .prepare(
            "SELECT before_verse, text FROM headings
             WHERE translation = ?1 AND book = ?2 AND chapter = ?3",
        )
        .ok()?;

    let headings: Vec<SectionHeading> = heading_stmt
        .query_map(rusqlite::params![translation_id, book, chapter], |row| {
            Ok(SectionHeading {
                before_verse: row.get(0)?,
                text: row.get(1)?,
            })
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();

    Some(Chapter {
        number: chapter,
        verses,
        headings,
    })
}
