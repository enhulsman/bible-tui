use quick_xml::events::Event as XmlEvent;
use quick_xml::reader::Reader;
use std::path::Path;

use crate::bible::model::{Chapter, Verse, VerseSpan};
use crate::import::{ImportedBook, ImportedTranslation};

/// Parse a Zefania XML Bible file.
/// Structure: <XMLBIBLE>/<BIBLEBOOK bnumber="1">/<CHAPTER cnumber="1">/<VERS vnumber="1">text</VERS>
pub fn parse(path: &Path) -> color_eyre::Result<ImportedTranslation> {
    let xml = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text_start = false;
    reader.config_mut().trim_text_end = false;

    let mut books: Vec<ImportedBook> = Vec::new();
    let mut current_book_num: Option<u8> = None;
    let mut current_book_name = String::new();
    let mut current_chapter: Option<u16> = None;
    let mut current_verse: Option<u8> = None;
    let mut verse_text = String::new();
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut verses: Vec<Verse> = Vec::new();

    let mut bible_name = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Eof) => break,
            Ok(XmlEvent::Start(ref e)) | Ok(XmlEvent::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_uppercase();
                match tag.as_str() {
                    "XMLBIBLE" => {
                        bible_name = get_attr(e, "biblename").unwrap_or_default();
                    }
                    "BIBLEBOOK" => {
                        // Finish previous book
                        if let Some(bn) = current_book_num {
                            finish_zef_verse(
                                &mut current_verse,
                                &mut verse_text,
                                &mut verses,
                            );
                            finish_zef_chapter(&mut current_chapter, &mut verses, &mut chapters);
                            if !chapters.is_empty() {
                                books.push(ImportedBook {
                                    number: bn,
                                    name: current_book_name.clone(),
                                    chapters: std::mem::take(&mut chapters),
                                });
                            }
                        }
                        current_book_num = get_attr(e, "bnumber")
                            .and_then(|s| s.parse().ok());
                        current_book_name =
                            get_attr(e, "bname").unwrap_or_else(|| "Unknown".to_string());
                    }
                    "CHAPTER" => {
                        finish_zef_verse(&mut current_verse, &mut verse_text, &mut verses);
                        finish_zef_chapter(&mut current_chapter, &mut verses, &mut chapters);
                        current_chapter = get_attr(e, "cnumber").and_then(|s| s.parse().ok());
                    }
                    "VERS" => {
                        finish_zef_verse(&mut current_verse, &mut verse_text, &mut verses);
                        current_verse = get_attr(e, "vnumber").and_then(|s| s.parse().ok());
                        verse_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_uppercase();
                match tag.as_str() {
                    "VERS" => {
                        finish_zef_verse(&mut current_verse, &mut verse_text, &mut verses);
                    }
                    "CHAPTER" => {
                        finish_zef_chapter(&mut current_chapter, &mut verses, &mut chapters);
                    }
                    "BIBLEBOOK" => {
                        finish_zef_verse(&mut current_verse, &mut verse_text, &mut verses);
                        finish_zef_chapter(&mut current_chapter, &mut verses, &mut chapters);
                        if let Some(bn) = current_book_num.take() {
                            if !chapters.is_empty() {
                                books.push(ImportedBook {
                                    number: bn,
                                    name: current_book_name.clone(),
                                    chapters: std::mem::take(&mut chapters),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::Text(ref e)) => {
                if current_verse.is_some() {
                    let text = e.unescape().unwrap_or_default().to_string();
                    verse_text.push_str(&text);
                }
            }
            Err(e) => {
                return Err(color_eyre::eyre::eyre!("Zefania XML parse error: {e}"));
            }
            _ => {}
        }
        buf.clear();
    }

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("import")
        .to_string()
        .to_lowercase()
        .replace(' ', "_");

    let name = if bible_name.is_empty() {
        id.clone()
    } else {
        bible_name
    };

    Ok(ImportedTranslation {
        id: id.clone(),
        name: name.clone(),
        abbreviation: id.chars().take(5).collect(),
        language: "unknown".to_string(),
        books,
    })
}

fn finish_zef_verse(
    current_verse: &mut Option<u8>,
    verse_text: &mut String,
    verses: &mut Vec<Verse>,
) {
    if let Some(vn) = current_verse.take() {
        let text = verse_text.trim().to_string();
        if !text.is_empty() {
            verses.push(Verse {
                number: vn,
                spans: vec![VerseSpan::Plain(text)],
                paragraph_break: false,
            });
        }
        verse_text.clear();
    }
}

fn finish_zef_chapter(
    current_chapter: &mut Option<u16>,
    verses: &mut Vec<Verse>,
    chapters: &mut Vec<Chapter>,
) {
    if let Some(cn) = current_chapter.take() {
        if !verses.is_empty() {
            chapters.push(Chapter {
                number: cn,
                verses: std::mem::take(verses),
                headings: vec![],
            });
        }
    }
    verses.clear();
}

fn get_attr(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| {
            let key = String::from_utf8_lossy(a.key.as_ref()).to_lowercase();
            key == name.to_lowercase()
        })
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
}
