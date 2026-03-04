use quick_xml::events::Event as XmlEvent;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Mirror the model types for build-time serialization.
// These must match src/bible/model.rs exactly.

#[derive(Serialize, Deserialize)]
enum TranslationId {
    Bundled(BundledTranslation),
    #[allow(dead_code)]
    Imported(String),
}

#[derive(Serialize, Deserialize)]
enum BundledTranslation {
    Kjv,
    Web,
    Sv,
}

#[derive(Serialize, Deserialize)]
enum VerseSpan {
    Plain(String),
    RedLetter(String),
    Selah,
}

#[derive(Serialize, Deserialize)]
struct Verse {
    number: u8,
    spans: Vec<VerseSpan>,
    paragraph_break: bool,
}

#[derive(Serialize, Deserialize)]
struct SectionHeading {
    text: String,
    before_verse: u8,
}

#[derive(Serialize, Deserialize)]
struct Chapter {
    number: u16,
    verses: Vec<Verse>,
    headings: Vec<SectionHeading>,
}

#[derive(Serialize, Deserialize)]
struct Book {
    name: String,
    code: String,
    index: u8,
    chapters: Vec<Chapter>,
}

#[derive(Serialize, Deserialize)]
struct BibleData {
    translation: TranslationId,
    books: Vec<Book>,
}

// KJV JSON structures
#[derive(Deserialize)]
struct KjvJson {
    #[allow(dead_code)]
    book: String,
    chapters: Vec<KjvChapter>,
}

#[derive(Deserialize)]
struct KjvChapter {
    chapter: String,
    verses: Vec<KjvVerse>,
}

#[derive(Deserialize)]
struct KjvVerse {
    verse: String,
    text: String,
}

// Canonical book order: (display_name, USFX/OSIS code, KJV filename)
const BOOKS: &[(&str, &str, &str)] = &[
    ("Genesis", "GEN", "Genesis"),
    ("Exodus", "EXO", "Exodus"),
    ("Leviticus", "LEV", "Leviticus"),
    ("Numbers", "NUM", "Numbers"),
    ("Deuteronomy", "DEU", "Deuteronomy"),
    ("Joshua", "JOS", "Joshua"),
    ("Judges", "JDG", "Judges"),
    ("Ruth", "RUT", "Ruth"),
    ("1 Samuel", "1SA", "1 Samuel"),
    ("2 Samuel", "2SA", "2 Samuel"),
    ("1 Kings", "1KI", "1 Kings"),
    ("2 Kings", "2KI", "2 Kings"),
    ("1 Chronicles", "1CH", "1 Chronicles"),
    ("2 Chronicles", "2CH", "2 Chronicles"),
    ("Ezra", "EZR", "Ezra"),
    ("Nehemiah", "NEH", "Nehemiah"),
    ("Esther", "EST", "Esther"),
    ("Job", "JOB", "Job"),
    ("Psalms", "PSA", "Psalms"),
    ("Proverbs", "PRO", "Proverbs"),
    ("Ecclesiastes", "ECC", "Ecclesiastes"),
    ("Song of Solomon", "SNG", "Song of Solomon"),
    ("Isaiah", "ISA", "Isaiah"),
    ("Jeremiah", "JER", "Jeremiah"),
    ("Lamentations", "LAM", "Lamentations"),
    ("Ezekiel", "EZK", "Ezekiel"),
    ("Daniel", "DAN", "Daniel"),
    ("Hosea", "HOS", "Hosea"),
    ("Joel", "JOL", "Joel"),
    ("Amos", "AMO", "Amos"),
    ("Obadiah", "OBA", "Obadiah"),
    ("Jonah", "JON", "Jonah"),
    ("Micah", "MIC", "Micah"),
    ("Nahum", "NAM", "Nahum"),
    ("Habakkuk", "HAB", "Habakkuk"),
    ("Zephaniah", "ZEP", "Zephaniah"),
    ("Haggai", "HAG", "Haggai"),
    ("Zechariah", "ZEC", "Zechariah"),
    ("Malachi", "MAL", "Malachi"),
    ("Matthew", "MAT", "Matthew"),
    ("Mark", "MRK", "Mark"),
    ("Luke", "LUK", "Luke"),
    ("John", "JHN", "John"),
    ("Acts", "ACT", "Acts"),
    ("Romans", "ROM", "Romans"),
    ("1 Corinthians", "1CO", "1 Corinthians"),
    ("2 Corinthians", "2CO", "2 Corinthians"),
    ("Galatians", "GAL", "Galatians"),
    ("Ephesians", "EPH", "Ephesians"),
    ("Philippians", "PHP", "Philippians"),
    ("Colossians", "COL", "Colossians"),
    ("1 Thessalonians", "1TH", "1 Thessalonians"),
    ("2 Thessalonians", "2TH", "2 Thessalonians"),
    ("1 Timothy", "1TI", "1 Timothy"),
    ("2 Timothy", "2TI", "2 Timothy"),
    ("Titus", "TIT", "Titus"),
    ("Philemon", "PHM", "Philemon"),
    ("Hebrews", "HEB", "Hebrews"),
    ("James", "JAS", "James"),
    ("1 Peter", "1PE", "1 Peter"),
    ("2 Peter", "2PE", "2 Peter"),
    ("1 John", "1JN", "1 John"),
    ("2 John", "2JN", "2 John"),
    ("3 John", "3JN", "3 John"),
    ("Jude", "JUD", "Jude"),
    ("Revelation", "REV", "Revelation"),
];

// OSIS book IDs used in the SV (map to our canonical codes)
fn osis_to_canon(osis_id: &str) -> Option<&'static str> {
    match osis_id {
        "Gen" => Some("GEN"),
        "Exod" => Some("EXO"),
        "Lev" => Some("LEV"),
        "Num" => Some("NUM"),
        "Deut" => Some("DEU"),
        "Josh" => Some("JOS"),
        "Judg" => Some("JDG"),
        "Ruth" => Some("RUT"),
        "1Sam" => Some("1SA"),
        "2Sam" => Some("2SA"),
        "1Kgs" => Some("1KI"),
        "2Kgs" => Some("2KI"),
        "1Chr" => Some("1CH"),
        "2Chr" => Some("2CH"),
        "Ezra" => Some("EZR"),
        "Neh" => Some("NEH"),
        "Esth" => Some("EST"),
        "Job" => Some("JOB"),
        "Ps" => Some("PSA"),
        "Prov" | "Pro" => Some("PRO"),
        "Eccl" => Some("ECC"),
        "Song" => Some("SNG"),
        "Isa" => Some("ISA"),
        "Jer" => Some("JER"),
        "Lam" => Some("LAM"),
        "Ezek" | "Eze" => Some("EZK"),
        "Dan" => Some("DAN"),
        "Hos" => Some("HOS"),
        "Joel" => Some("JOL"),
        "Amos" | "Amo" => Some("AMO"),
        "Obad" | "Oba" => Some("OBA"),
        "Jonah" | "Jon" => Some("JON"),
        "Mic" => Some("MIC"),
        "Nah" => Some("NAM"),
        "Hab" => Some("HAB"),
        "Zeph" | "Zep" => Some("ZEP"),
        "Hag" => Some("HAG"),
        "Zech" | "Zec" => Some("ZEC"),
        "Mal" => Some("MAL"),
        "Matt" | "Mat" => Some("MAT"),
        "Mark" | "Mar" => Some("MRK"),
        "Luke" | "Luk" => Some("LUK"),
        "John" | "Joh" => Some("JHN"),
        "Acts" | "Act" => Some("ACT"),
        "Rom" => Some("ROM"),
        "1Cor" => Some("1CO"),
        "2Cor" => Some("2CO"),
        "Gal" => Some("GAL"),
        "Eph" => Some("EPH"),
        "Phil" | "Php" => Some("PHP"),
        "Col" => Some("COL"),
        "1Thess" | "1Th" => Some("1TH"),
        "2Thess" | "2Th" => Some("2TH"),
        "1Tim" | "1Ti" => Some("1TI"),
        "2Tim" | "2Ti" => Some("2TI"),
        "Titus" | "Tit" => Some("TIT"),
        "Phlm" | "Phm" => Some("PHM"),
        "Heb" => Some("HEB"),
        "Jas" => Some("JAS"),
        "1Pet" | "1Pe" => Some("1PE"),
        "2Pet" | "2Pe" => Some("2PE"),
        "1John" | "1Jn" => Some("1JN"),
        "2John" | "2Jn" => Some("2JN"),
        "3John" | "3Jn" => Some("3JN"),
        "Jude" | "Jud" => Some("JUD"),
        "Rev" => Some("REV"),
        _ => None,
    }
}

// SV Dutch book names
fn sv_book_name(code: &str) -> &'static str {
    match code {
        "GEN" => "Genesis",
        "EXO" => "Exodus",
        "LEV" => "Leviticus",
        "NUM" => "Numeri",
        "DEU" => "Deuteronomium",
        "JOS" => "Jozua",
        "JDG" => "Richteren",
        "RUT" => "Ruth",
        "1SA" => "1 Samuël",
        "2SA" => "2 Samuël",
        "1KI" => "1 Koningen",
        "2KI" => "2 Koningen",
        "1CH" => "1 Kronieken",
        "2CH" => "2 Kronieken",
        "EZR" => "Ezra",
        "NEH" => "Nehemia",
        "EST" => "Esther",
        "JOB" => "Job",
        "PSA" => "Psalmen",
        "PRO" => "Spreuken",
        "ECC" => "Prediker",
        "SNG" => "Hooglied",
        "ISA" => "Jesaja",
        "JER" => "Jeremia",
        "LAM" => "Klaagliederen",
        "EZK" => "Ezechiël",
        "DAN" => "Daniël",
        "HOS" => "Hosea",
        "JOL" => "Joël",
        "AMO" => "Amos",
        "OBA" => "Obadja",
        "JON" => "Jona",
        "MIC" => "Micha",
        "NAM" => "Nahum",
        "HAB" => "Habakuk",
        "ZEP" => "Zefanja",
        "HAG" => "Haggaï",
        "ZEC" => "Zacharia",
        "MAL" => "Maleachi",
        "MAT" => "Mattheüs",
        "MRK" => "Markus",
        "LUK" => "Lukas",
        "JHN" => "Johannes",
        "ACT" => "Handelingen",
        "ROM" => "Romeinen",
        "1CO" => "1 Korinthe",
        "2CO" => "2 Korinthe",
        "GAL" => "Galaten",
        "EPH" => "Efeze",
        "PHP" => "Filippenzen",
        "COL" => "Kolossenzen",
        "1TH" => "1 Thessalonicenzen",
        "2TH" => "2 Thessalonicenzen",
        "1TI" => "1 Timotheüs",
        "2TI" => "2 Timotheüs",
        "TIT" => "Titus",
        "PHM" => "Filemon",
        "HEB" => "Hebreeën",
        "JAS" => "Jakobus",
        "1PE" => "1 Petrus",
        "2PE" => "2 Petrus",
        "1JN" => "1 Johannes",
        "2JN" => "2 Johannes",
        "3JN" => "3 Johannes",
        "JUD" => "Judas",
        "REV" => "Openbaring",
        _ => "?",
    }
}

fn build_kjv(out_dir: &str) {
    println!("cargo:rerun-if-changed=data/sources/kjv");
    let kjv_dir = Path::new("data/sources/kjv");
    let mut books = Vec::with_capacity(66);

    for (index, &(name, code, filename)) in BOOKS.iter().enumerate() {
        let path = kjv_dir.join(format!("{filename}.json"));
        let json_str = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let kjv: KjvJson = serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

        let chapters: Vec<Chapter> = kjv
            .chapters
            .into_iter()
            .map(|ch| {
                let chapter_num: u16 = ch.chapter.parse().unwrap();
                let verses: Vec<Verse> = ch
                    .verses
                    .into_iter()
                    .map(|v| Verse {
                        number: v.verse.parse().unwrap(),
                        spans: vec![VerseSpan::Plain(v.text)],
                        paragraph_break: false,
                    })
                    .collect();
                Chapter {
                    number: chapter_num,
                    verses,
                    headings: vec![],
                }
            })
            .collect();

        books.push(Book {
            name: name.to_string(),
            code: code.to_string(),
            index: index as u8,
            chapters,
        });
    }

    let bible = BibleData {
        translation: TranslationId::Bundled(BundledTranslation::Kjv),
        books,
    };
    let encoded = postcard::to_allocvec(&bible).expect("Failed to serialize KJV");
    let out_path = Path::new(out_dir).join("kjv.postcard");
    fs::write(&out_path, &encoded).expect("Failed to write KJV postcard");
    println!(
        "cargo:warning=KJV postcard: {} bytes ({:.1} MB)",
        encoded.len(),
        encoded.len() as f64 / 1_048_576.0
    );
}

fn build_web(out_dir: &str) {
    println!("cargo:rerun-if-changed=data/sources/web");
    let path = Path::new("data/sources/web/eng-web.usfx.xml");
    let xml = fs::read_to_string(path).expect("Failed to read WEB USFX");

    // Parse USFX: <book id="GEN">, <c id="1">, <v id="1">, <wj> for red-letter, <s> for headings
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text_start = false;
    reader.config_mut().trim_text_end = false;

    let mut books_map: HashMap<String, Vec<Chapter>> = HashMap::new();
    let mut current_book: Option<String> = None;
    let mut current_chapter: Option<u16> = None;
    let mut current_verse: Option<u8> = None;
    let mut current_spans: Vec<VerseSpan> = Vec::new();
    let mut in_wj = false; // red-letter (words of Jesus)
    let mut in_note = false; // skip footnote content
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut verses: Vec<Verse> = Vec::new();
    let mut headings: Vec<SectionHeading> = Vec::new();
    let mut pending_heading: Option<String> = None;
    let mut in_heading = false;
    let mut heading_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Eof) => break,
            Ok(XmlEvent::Start(ref e)) | Ok(XmlEvent::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "book" => {
                        // Finish previous book
                        if let Some(ref book_code) = current_book {
                            finish_verse(
                                &mut current_verse,
                                &mut current_spans,
                                &mut verses,
                            );
                            finish_chapter(
                                &mut current_chapter,
                                &mut verses,
                                &mut headings,
                                &mut chapters,
                            );
                            if !chapters.is_empty() {
                                books_map.insert(book_code.clone(), std::mem::take(&mut chapters));
                            }
                        }
                        let id = get_attr(e, "id");
                        current_book = id;
                        current_chapter = None;
                        current_verse = None;
                    }
                    "c" => {
                        finish_verse(
                            &mut current_verse,
                            &mut current_spans,
                            &mut verses,
                        );
                        finish_chapter(
                            &mut current_chapter,
                            &mut verses,
                            &mut headings,
                            &mut chapters,
                        );
                        if let Some(id) = get_attr(e, "id") {
                            current_chapter = id.parse().ok();
                        }
                    }
                    "v" => {
                        finish_verse(
                            &mut current_verse,
                            &mut current_spans,
                            &mut verses,
                        );
                        if let Some(ref h) = pending_heading {
                            let next_verse: u8 = get_attr(e, "id")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(1);
                            headings.push(SectionHeading {
                                text: h.clone(),
                                before_verse: next_verse,
                            });
                            pending_heading = None;
                        }
                        if let Some(id) = get_attr(e, "id") {
                            current_verse = id.parse().ok();
                        }
                    }
                    "wj" => in_wj = true,
                    "f" | "x" | "note" => in_note = true,
                    "s" => {
                        in_heading = true;
                        heading_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "wj" => in_wj = false,
                    "f" | "x" | "note" => in_note = false,
                    "s" => {
                        in_heading = false;
                        if !heading_text.trim().is_empty() {
                            pending_heading = Some(heading_text.trim().to_string());
                        }
                    }
                    "book" => {
                        if let Some(ref book_code) = current_book {
                            finish_verse(
                                &mut current_verse,
                                &mut current_spans,
                                &mut verses,
                            );
                            finish_chapter(
                                &mut current_chapter,
                                &mut verses,
                                &mut headings,
                                &mut chapters,
                            );
                            if !chapters.is_empty() {
                                books_map.insert(book_code.clone(), std::mem::take(&mut chapters));
                            }
                            current_book = None;
                        }
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::Text(ref e)) => {
                if in_note {
                    continue;
                }
                let text = e.unescape().unwrap_or_default().to_string();
                if in_heading {
                    heading_text.push_str(&text);
                } else if current_verse.is_some() && !text.is_empty() {
                    if in_wj {
                        current_spans.push(VerseSpan::RedLetter(text));
                    } else {
                        current_spans.push(VerseSpan::Plain(text));
                    }
                }
            }
            Err(e) => panic!("USFX parse error: {e}"),
            _ => {}
        }
        buf.clear();
    }

    // Assemble books in canonical order
    let mut books = Vec::with_capacity(66);
    for (index, &(name, code, _)) in BOOKS.iter().enumerate() {
        if let Some(chs) = books_map.remove(code) {
            books.push(Book {
                name: name.to_string(),
                code: code.to_string(),
                index: index as u8,
                chapters: chs,
            });
        }
    }

    let bible = BibleData {
        translation: TranslationId::Bundled(BundledTranslation::Web),
        books,
    };
    let encoded = postcard::to_allocvec(&bible).expect("Failed to serialize WEB");
    let out_path = Path::new(out_dir).join("web.postcard");
    fs::write(&out_path, &encoded).expect("Failed to write WEB postcard");
    println!(
        "cargo:warning=WEB postcard: {} bytes ({:.1} MB), {} books",
        encoded.len(),
        encoded.len() as f64 / 1_048_576.0,
        bible.books.len()
    );
}

fn build_sv(out_dir: &str) {
    println!("cargo:rerun-if-changed=data/sources/sv");
    let path = Path::new("data/sources/sv/STV.xml");
    let xml = fs::read_to_string(path).expect("Failed to read SV OSIS XML");

    // Parse OSIS: <div osisID="Gen" type="book">, <chapter osisID="Gen.1">,
    //             <verse osisID="Gen.1.1">, <w lemma="...">, <note>
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text_start = false;
    reader.config_mut().trim_text_end = false;

    let mut books_map: HashMap<String, Vec<Chapter>> = HashMap::new();
    let mut current_book_code: Option<String> = None;
    let mut current_chapter: Option<u16> = None;
    let mut current_verse: Option<u8> = None;
    let mut current_spans: Vec<VerseSpan> = Vec::new();
    let mut in_note = false;
    let mut note_depth: u32 = 0;
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut verses: Vec<Verse> = Vec::new();
    let mut headings: Vec<SectionHeading> = Vec::new();
    let mut title_text = String::new();
    let mut in_title = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Eof) => break,
            Ok(XmlEvent::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if in_note {
                    note_depth += 1;
                    buf.clear();
                    continue;
                }

                match tag.as_str() {
                    "div" => {
                        let div_type = get_attr(e, "type").unwrap_or_default();
                        if div_type == "book" {
                            // Finish previous book
                            if let Some(ref code) = current_book_code {
                                finish_verse(
                                    &mut current_verse,
                                    &mut current_spans,
                                    &mut verses,
                                );
                                finish_chapter(
                                    &mut current_chapter,
                                    &mut verses,
                                    &mut headings,
                                    &mut chapters,
                                );
                                if !chapters.is_empty() {
                                    books_map
                                        .insert(code.clone(), std::mem::take(&mut chapters));
                                }
                            }
                            if let Some(osis_id) = get_attr(e, "osisID") {
                                current_book_code =
                                    osis_to_canon(&osis_id).map(|s| s.to_string());
                            }
                        }
                    }
                    "chapter" => {
                        finish_verse(
                            &mut current_verse,
                            &mut current_spans,
                            &mut verses,
                        );
                        finish_chapter(
                            &mut current_chapter,
                            &mut verses,
                            &mut headings,
                            &mut chapters,
                        );
                        // osisID="Gen.1" → chapter 1
                        if let Some(osis_id) = get_attr(e, "osisID") {
                            current_chapter = osis_id
                                .rsplit('.')
                                .next()
                                .and_then(|s| s.parse().ok());
                        }
                    }
                    "verse" => {
                        finish_verse(
                            &mut current_verse,
                            &mut current_spans,
                            &mut verses,
                        );
                        // osisID="Gen.1.1" → verse 1
                        if let Some(osis_id) = get_attr(e, "osisID") {
                            // Handle ranges like "Gen.1.1-Gen.1.2" — take first verse
                            let first = osis_id.split('-').next().unwrap_or(&osis_id);
                            current_verse = first
                                .rsplit('.')
                                .next()
                                .and_then(|s| s.parse().ok());
                        }
                    }
                    "note" => {
                        in_note = true;
                        note_depth = 1;
                    }
                    "title" => {
                        in_title = true;
                        title_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if in_note {
                    note_depth -= 1;
                    if note_depth == 0 {
                        in_note = false;
                    }
                    buf.clear();
                    continue;
                }

                match tag.as_str() {
                    "title" => {
                        in_title = false;
                        // Store as heading before next verse
                        if !title_text.trim().is_empty() && current_chapter.is_some() {
                            let next_verse = (verses.len() as u8) + 1;
                            headings.push(SectionHeading {
                                text: title_text.trim().to_string(),
                                before_verse: next_verse,
                            });
                        }
                    }
                    "div" => {
                        // Could be end of book
                        if let Some(ref code) = current_book_code {
                            // Check if this closes a book div by seeing if we have data
                            // We'll handle this via the next book opening
                            let _ = code;
                        }
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::Empty(ref e)) => {
                if in_note {
                    buf.clear();
                    continue;
                }
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "verse" {
                    // Self-closing verse end marker (eID), finish current verse
                    finish_verse(
                        &mut current_verse,
                        &mut current_spans,
                        &mut verses,
                    );
                } else if tag == "chapter" {
                    // Self-closing chapter end marker
                    finish_verse(
                        &mut current_verse,
                        &mut current_spans,
                        &mut verses,
                    );
                    finish_chapter(
                        &mut current_chapter,
                        &mut verses,
                        &mut headings,
                        &mut chapters,
                    );
                }
            }
            Ok(XmlEvent::Text(ref e)) => {
                if in_note {
                    buf.clear();
                    continue;
                }
                let text = e.unescape().unwrap_or_default().to_string();
                if in_title {
                    title_text.push_str(&text);
                } else if current_verse.is_some() && !text.is_empty() {
                    current_spans.push(VerseSpan::Plain(text));
                }
            }
            Err(e) => panic!("OSIS parse error at position {}: {e}", reader.buffer_position()),
            _ => {}
        }
        buf.clear();
    }

    // Finish last book
    if let Some(ref code) = current_book_code {
        finish_verse(&mut current_verse, &mut current_spans, &mut verses);
        finish_chapter(
            &mut current_chapter,
            &mut verses,
            &mut headings,
            &mut chapters,
        );
        if !chapters.is_empty() {
            books_map.insert(code.clone(), std::mem::take(&mut chapters));
        }
    }

    // Assemble in canonical order with Dutch names
    let mut books = Vec::with_capacity(66);
    for (index, &(_, code, _)) in BOOKS.iter().enumerate() {
        if let Some(chs) = books_map.remove(code) {
            books.push(Book {
                name: sv_book_name(code).to_string(),
                code: code.to_string(),
                index: index as u8,
                chapters: chs,
            });
        }
    }

    let bible = BibleData {
        translation: TranslationId::Bundled(BundledTranslation::Sv),
        books,
    };
    let encoded = postcard::to_allocvec(&bible).expect("Failed to serialize SV");
    let out_path = Path::new(out_dir).join("sv.postcard");
    fs::write(&out_path, &encoded).expect("Failed to write SV postcard");
    println!(
        "cargo:warning=SV postcard: {} bytes ({:.1} MB), {} books",
        encoded.len(),
        encoded.len() as f64 / 1_048_576.0,
        bible.books.len()
    );
}

fn finish_verse(
    current_verse: &mut Option<u8>,
    current_spans: &mut Vec<VerseSpan>,
    verses: &mut Vec<Verse>,
) {
    if let Some(vn) = current_verse.take() {
        if !current_spans.is_empty() {
            // Merge adjacent spans of the same type and clean up whitespace
            let spans = consolidate_spans(std::mem::take(current_spans));
            verses.push(Verse {
                number: vn,
                spans,
                paragraph_break: false,
            });
        }
        current_spans.clear();
    }
}

fn finish_chapter(
    current_chapter: &mut Option<u16>,
    verses: &mut Vec<Verse>,
    headings: &mut Vec<SectionHeading>,
    chapters: &mut Vec<Chapter>,
) {
    if let Some(cn) = current_chapter.take() {
        if !verses.is_empty() {
            chapters.push(Chapter {
                number: cn,
                verses: std::mem::take(verses),
                headings: std::mem::take(headings),
            });
        }
    }
    verses.clear();
    headings.clear();
}

fn consolidate_spans(spans: Vec<VerseSpan>) -> Vec<VerseSpan> {
    let mut result: Vec<VerseSpan> = Vec::new();
    for span in spans {
        match (&mut result.last_mut(), &span) {
            (Some(VerseSpan::Plain(ref mut existing)), VerseSpan::Plain(new)) => {
                existing.push_str(new);
            }
            (Some(VerseSpan::RedLetter(ref mut existing)), VerseSpan::RedLetter(new)) => {
                existing.push_str(new);
            }
            _ => result.push(span),
        }
    }
    // Normalize whitespace: collapse \n, \r, \t, and multiple spaces into single space
    for span in &mut result {
        let text = match span {
            VerseSpan::Plain(t) | VerseSpan::RedLetter(t) => t,
            _ => continue,
        };
        *text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    result
}

fn get_attr(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == name.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    build_kjv(&out_dir);
    build_web(&out_dir);
    build_sv(&out_dir);
}
