use serde::Deserialize;
use std::path::Path;

use crate::bible::model::{Chapter, Verse, VerseSpan};
use crate::import::{ImportedBook, ImportedTranslation};

/// Simple JSON Bible format:
/// {
///   "name": "Translation Name",
///   "abbreviation": "TN",
///   "language": "en",
///   "books": [
///     { "name": "Genesis", "number": 1, "chapters": [
///       { "chapter": 1, "verses": [
///         { "verse": 1, "text": "In the beginning..." }
///       ]}
///     ]}
///   ]
/// }
#[derive(Deserialize)]
struct JsonBible {
    name: Option<String>,
    abbreviation: Option<String>,
    language: Option<String>,
    books: Vec<JsonBook>,
}

#[derive(Deserialize)]
struct JsonBook {
    name: String,
    number: Option<u8>,
    chapters: Vec<JsonChapter>,
}

#[derive(Deserialize)]
struct JsonChapter {
    chapter: u16,
    verses: Vec<JsonVerse>,
}

#[derive(Deserialize)]
struct JsonVerse {
    verse: u8,
    text: String,
}

pub fn parse(path: &Path) -> color_eyre::Result<ImportedTranslation> {
    let content = std::fs::read_to_string(path)?;
    let data: JsonBible = serde_json::from_str(&content)?;

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("import")
        .to_string()
        .to_lowercase()
        .replace(' ', "_");

    let name = data.name.unwrap_or_else(|| id.clone());
    let abbreviation = data
        .abbreviation
        .unwrap_or_else(|| id.chars().take(5).collect());
    let language = data.language.unwrap_or_else(|| "unknown".to_string());

    let books: Vec<ImportedBook> = data
        .books
        .into_iter()
        .enumerate()
        .map(|(i, book)| {
            let chapters: Vec<Chapter> = book
                .chapters
                .into_iter()
                .map(|ch| Chapter {
                    number: ch.chapter,
                    verses: ch
                        .verses
                        .into_iter()
                        .map(|v| Verse {
                            number: v.verse,
                            spans: vec![VerseSpan::Plain(v.text)],
                            paragraph_break: false,
                        })
                        .collect(),
                    headings: vec![],
                })
                .collect();

            ImportedBook {
                number: book.number.unwrap_or((i + 1) as u8),
                name: book.name,
                chapters,
            }
        })
        .collect();

    Ok(ImportedTranslation {
        id,
        name,
        abbreviation,
        language,
        books,
    })
}
