pub mod json;
pub mod mybible;
pub mod zefania;

use std::path::Path;

use crate::bible::model::{Chapter, SectionHeading, Verse, VerseSpan};

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
#[allow(dead_code)]
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
