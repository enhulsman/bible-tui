use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BundledTranslation {
    Kjv,
    Web,
    Sv,
}

impl BundledTranslation {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::Kjv => "KJV",
            Self::Web => "WEB",
            Self::Sv => "SV",
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kjv => "King James Version",
            Self::Web => "World English Bible",
            Self::Sv => "Statenvertaling",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TranslationId {
    Bundled(BundledTranslation),
    Imported(String),
}

impl std::fmt::Display for TranslationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundled(b) => write!(f, "{}", b.abbreviation()),
            Self::Imported(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerseSpan {
    Plain(String),
    RedLetter(String),
    Selah,
}

impl VerseSpan {
    pub fn text(&self) -> &str {
        match self {
            Self::Plain(s) | Self::RedLetter(s) => s,
            Self::Selah => "Selah",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub number: u8,
    pub spans: Vec<VerseSpan>,
    pub paragraph_break: bool,
}

impl Verse {
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text()).collect::<Vec<_>>().join("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionHeading {
    pub text: String,
    pub before_verse: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub number: u16,
    pub verses: Vec<Verse>,
    pub headings: Vec<SectionHeading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name: String,
    pub code: String,
    pub index: u8,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibleData {
    pub translation: TranslationId,
    pub books: Vec<Book>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerseRef {
    pub book_index: u8,
    pub chapter: u16,
    pub verse: u8,
}

impl std::fmt::Display for VerseRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.chapter, self.verse)
    }
}
