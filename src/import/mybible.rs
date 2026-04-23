use std::path::Path;

use crate::bible::model::{Chapter, Verse, VerseSpan};
use crate::import::{ImportedBook, ImportedTranslation};

/// Parse a MyBible SQLite database.
/// MyBible format: table `verses` with columns book_number, chapter, verse, text
/// Table `info` with key-value pairs for metadata.
pub fn parse(path: &Path) -> color_eyre::Result<ImportedTranslation> {
    let db = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;

    // Read metadata from info table
    let mut name = String::new();
    let mut abbreviation = String::new();
    let mut language = String::from("unknown");

    if let Ok(mut stmt) = db.prepare("SELECT name, value FROM info") {
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap_or_else(|_| panic!("Failed to read info table"))
            .filter_map(|r| r.ok())
            .collect();

        for (key, value) in &rows {
            match key.as_str() {
                "description" | "detailed_info" => {
                    if name.is_empty() {
                        name = value.clone();
                    }
                }
                "abbreviation" => abbreviation = value.clone(),
                "language" => language = value.clone(),
                _ => {}
            }
        }
    }

    if name.is_empty() {
        name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
    }
    if abbreviation.is_empty() {
        abbreviation = name.chars().take(5).collect();
    }

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("import")
        .to_string()
        .to_lowercase()
        .replace(' ', "_");

    // Read verses
    let mut stmt = db.prepare(
        "SELECT book_number, chapter, verse, text FROM verses ORDER BY book_number, chapter, verse",
    )?;

    let mut books_map: std::collections::BTreeMap<u8, Vec<(u16, u8, String)>> =
        std::collections::BTreeMap::new();

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)? as u8,
            row.get::<_, i32>(1)? as u16,
            row.get::<_, i32>(2)? as u8,
            row.get::<_, String>(3)?,
        ))
    })?;

    for row in rows {
        let (book_num, chapter, verse, text) = row?;
        // Strip HTML tags from MyBible text
        let clean_text = strip_html(&text);
        books_map
            .entry(book_num)
            .or_default()
            .push((chapter, verse, clean_text));
    }

    let mut books = Vec::new();
    for (book_num, verse_data) in books_map {
        let mut chapters_map: std::collections::BTreeMap<u16, Vec<Verse>> =
            std::collections::BTreeMap::new();

        for (ch_num, v_num, text) in verse_data {
            chapters_map.entry(ch_num).or_default().push(Verse {
                number: v_num,
                spans: vec![VerseSpan::Plain(text)],
                paragraph_break: false,
            });
        }

        let chapters: Vec<Chapter> = chapters_map
            .into_iter()
            .map(|(num, verses)| Chapter {
                number: num,
                verses,
                headings: vec![],
            })
            .collect();

        let book_name = mybible_book_name(book_num);
        books.push(ImportedBook {
            number: book_num,
            name: book_name.to_string(),
            chapters,
        });
    }

    Ok(ImportedTranslation {
        id,
        name,
        abbreviation,
        language,
        books,
    })
}

fn strip_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

fn mybible_book_name(num: u8) -> &'static str {
    match num {
        1 => "Genesis", 2 => "Exodus", 3 => "Leviticus", 4 => "Numbers",
        5 => "Deuteronomy", 6 => "Joshua", 7 => "Judges", 8 => "Ruth",
        9 => "1 Samuel", 10 => "2 Samuel", 11 => "1 Kings", 12 => "2 Kings",
        13 => "1 Chronicles", 14 => "2 Chronicles", 15 => "Ezra", 16 => "Nehemiah",
        17 => "Esther", 18 => "Job", 19 => "Psalms", 20 => "Proverbs",
        21 => "Ecclesiastes", 22 => "Song of Solomon", 23 => "Isaiah", 24 => "Jeremiah",
        25 => "Lamentations", 26 => "Ezekiel", 27 => "Daniel", 28 => "Hosea",
        29 => "Joel", 30 => "Amos", 31 => "Obadiah", 32 => "Jonah",
        33 => "Micah", 34 => "Nahum", 35 => "Habakkuk", 36 => "Zephaniah",
        37 => "Haggai", 38 => "Zechariah", 39 => "Malachi",
        40 => "Matthew", 41 => "Mark", 42 => "Luke", 43 => "John",
        44 => "Acts", 45 => "Romans", 46 => "1 Corinthians", 47 => "2 Corinthians",
        48 => "Galatians", 49 => "Ephesians", 50 => "Philippians", 51 => "Colossians",
        52 => "1 Thessalonians", 53 => "2 Thessalonians", 54 => "1 Timothy",
        55 => "2 Timothy", 56 => "Titus", 57 => "Philemon", 58 => "Hebrews",
        59 => "James", 60 => "1 Peter", 61 => "2 Peter", 62 => "1 John",
        63 => "2 John", 64 => "3 John", 65 => "Jude", 66 => "Revelation",
        _ => "Unknown",
    }
}
