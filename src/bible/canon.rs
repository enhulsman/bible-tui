/// Canonical book info for the 66-book Protestant canon.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BookInfo {
    pub name: &'static str,
    pub abbreviation: &'static str,
    pub code: &'static str,
    pub chapter_count: u16,
    pub testament: Testament,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Testament {
    Old,
    New,
}

/// All 66 books in canonical order.
pub const CANON: [BookInfo; 66] = [
    BookInfo { name: "Genesis", abbreviation: "Gen", code: "GEN", chapter_count: 50, testament: Testament::Old },
    BookInfo { name: "Exodus", abbreviation: "Exo", code: "EXO", chapter_count: 40, testament: Testament::Old },
    BookInfo { name: "Leviticus", abbreviation: "Lev", code: "LEV", chapter_count: 27, testament: Testament::Old },
    BookInfo { name: "Numbers", abbreviation: "Num", code: "NUM", chapter_count: 36, testament: Testament::Old },
    BookInfo { name: "Deuteronomy", abbreviation: "Deu", code: "DEU", chapter_count: 34, testament: Testament::Old },
    BookInfo { name: "Joshua", abbreviation: "Jos", code: "JOS", chapter_count: 24, testament: Testament::Old },
    BookInfo { name: "Judges", abbreviation: "Jdg", code: "JDG", chapter_count: 21, testament: Testament::Old },
    BookInfo { name: "Ruth", abbreviation: "Rut", code: "RUT", chapter_count: 4, testament: Testament::Old },
    BookInfo { name: "1 Samuel", abbreviation: "1Sa", code: "1SA", chapter_count: 31, testament: Testament::Old },
    BookInfo { name: "2 Samuel", abbreviation: "2Sa", code: "2SA", chapter_count: 24, testament: Testament::Old },
    BookInfo { name: "1 Kings", abbreviation: "1Ki", code: "1KI", chapter_count: 22, testament: Testament::Old },
    BookInfo { name: "2 Kings", abbreviation: "2Ki", code: "2KI", chapter_count: 25, testament: Testament::Old },
    BookInfo { name: "1 Chronicles", abbreviation: "1Ch", code: "1CH", chapter_count: 29, testament: Testament::Old },
    BookInfo { name: "2 Chronicles", abbreviation: "2Ch", code: "2CH", chapter_count: 36, testament: Testament::Old },
    BookInfo { name: "Ezra", abbreviation: "Ezr", code: "EZR", chapter_count: 10, testament: Testament::Old },
    BookInfo { name: "Nehemiah", abbreviation: "Neh", code: "NEH", chapter_count: 13, testament: Testament::Old },
    BookInfo { name: "Esther", abbreviation: "Est", code: "EST", chapter_count: 10, testament: Testament::Old },
    BookInfo { name: "Job", abbreviation: "Job", code: "JOB", chapter_count: 42, testament: Testament::Old },
    BookInfo { name: "Psalms", abbreviation: "Psa", code: "PSA", chapter_count: 150, testament: Testament::Old },
    BookInfo { name: "Proverbs", abbreviation: "Pro", code: "PRO", chapter_count: 31, testament: Testament::Old },
    BookInfo { name: "Ecclesiastes", abbreviation: "Ecc", code: "ECC", chapter_count: 12, testament: Testament::Old },
    BookInfo { name: "Song of Solomon", abbreviation: "Sol", code: "SNG", chapter_count: 8, testament: Testament::Old },
    BookInfo { name: "Isaiah", abbreviation: "Isa", code: "ISA", chapter_count: 66, testament: Testament::Old },
    BookInfo { name: "Jeremiah", abbreviation: "Jer", code: "JER", chapter_count: 52, testament: Testament::Old },
    BookInfo { name: "Lamentations", abbreviation: "Lam", code: "LAM", chapter_count: 5, testament: Testament::Old },
    BookInfo { name: "Ezekiel", abbreviation: "Eze", code: "EZK", chapter_count: 48, testament: Testament::Old },
    BookInfo { name: "Daniel", abbreviation: "Dan", code: "DAN", chapter_count: 12, testament: Testament::Old },
    BookInfo { name: "Hosea", abbreviation: "Hos", code: "HOS", chapter_count: 14, testament: Testament::Old },
    BookInfo { name: "Joel", abbreviation: "Joe", code: "JOL", chapter_count: 3, testament: Testament::Old },
    BookInfo { name: "Amos", abbreviation: "Amo", code: "AMO", chapter_count: 9, testament: Testament::Old },
    BookInfo { name: "Obadiah", abbreviation: "Oba", code: "OBA", chapter_count: 1, testament: Testament::Old },
    BookInfo { name: "Jonah", abbreviation: "Jon", code: "JNH", chapter_count: 4, testament: Testament::Old },
    BookInfo { name: "Micah", abbreviation: "Mic", code: "MIC", chapter_count: 7, testament: Testament::Old },
    BookInfo { name: "Nahum", abbreviation: "Nah", code: "NAM", chapter_count: 3, testament: Testament::Old },
    BookInfo { name: "Habakkuk", abbreviation: "Hab", code: "HAB", chapter_count: 3, testament: Testament::Old },
    BookInfo { name: "Zephaniah", abbreviation: "Zep", code: "ZEP", chapter_count: 3, testament: Testament::Old },
    BookInfo { name: "Haggai", abbreviation: "Hag", code: "HAG", chapter_count: 2, testament: Testament::Old },
    BookInfo { name: "Zechariah", abbreviation: "Zec", code: "ZEC", chapter_count: 14, testament: Testament::Old },
    BookInfo { name: "Malachi", abbreviation: "Mal", code: "MAL", chapter_count: 4, testament: Testament::Old },
    BookInfo { name: "Matthew", abbreviation: "Mat", code: "MAT", chapter_count: 28, testament: Testament::New },
    BookInfo { name: "Mark", abbreviation: "Mar", code: "MRK", chapter_count: 16, testament: Testament::New },
    BookInfo { name: "Luke", abbreviation: "Luk", code: "LUK", chapter_count: 24, testament: Testament::New },
    BookInfo { name: "John", abbreviation: "Joh", code: "JHN", chapter_count: 21, testament: Testament::New },
    BookInfo { name: "Acts", abbreviation: "Act", code: "ACT", chapter_count: 28, testament: Testament::New },
    BookInfo { name: "Romans", abbreviation: "Rom", code: "ROM", chapter_count: 16, testament: Testament::New },
    BookInfo { name: "1 Corinthians", abbreviation: "1Co", code: "1CO", chapter_count: 16, testament: Testament::New },
    BookInfo { name: "2 Corinthians", abbreviation: "2Co", code: "2CO", chapter_count: 13, testament: Testament::New },
    BookInfo { name: "Galatians", abbreviation: "Gal", code: "GAL", chapter_count: 6, testament: Testament::New },
    BookInfo { name: "Ephesians", abbreviation: "Eph", code: "EPH", chapter_count: 6, testament: Testament::New },
    BookInfo { name: "Philippians", abbreviation: "Phi", code: "PHP", chapter_count: 4, testament: Testament::New },
    BookInfo { name: "Colossians", abbreviation: "Col", code: "COL", chapter_count: 4, testament: Testament::New },
    BookInfo { name: "1 Thessalonians", abbreviation: "1Th", code: "1TH", chapter_count: 5, testament: Testament::New },
    BookInfo { name: "2 Thessalonians", abbreviation: "2Th", code: "2TH", chapter_count: 3, testament: Testament::New },
    BookInfo { name: "1 Timothy", abbreviation: "1Ti", code: "1TI", chapter_count: 6, testament: Testament::New },
    BookInfo { name: "2 Timothy", abbreviation: "2Ti", code: "2TI", chapter_count: 4, testament: Testament::New },
    BookInfo { name: "Titus", abbreviation: "Tit", code: "TIT", chapter_count: 3, testament: Testament::New },
    BookInfo { name: "Philemon", abbreviation: "Phm", code: "PHM", chapter_count: 1, testament: Testament::New },
    BookInfo { name: "Hebrews", abbreviation: "Heb", code: "HEB", chapter_count: 13, testament: Testament::New },
    BookInfo { name: "James", abbreviation: "Jam", code: "JAS", chapter_count: 5, testament: Testament::New },
    BookInfo { name: "1 Peter", abbreviation: "1Pe", code: "1PE", chapter_count: 5, testament: Testament::New },
    BookInfo { name: "2 Peter", abbreviation: "2Pe", code: "2PE", chapter_count: 3, testament: Testament::New },
    BookInfo { name: "1 John", abbreviation: "1Jo", code: "1JN", chapter_count: 5, testament: Testament::New },
    BookInfo { name: "2 John", abbreviation: "2Jo", code: "2JN", chapter_count: 1, testament: Testament::New },
    BookInfo { name: "3 John", abbreviation: "3Jo", code: "3JN", chapter_count: 1, testament: Testament::New },
    BookInfo { name: "Jude", abbreviation: "Jud", code: "JUD", chapter_count: 1, testament: Testament::New },
    BookInfo { name: "Revelation", abbreviation: "Rev", code: "REV", chapter_count: 22, testament: Testament::New },
];

/// Find a book index by name, abbreviation, or code (case-insensitive).
pub fn find_book(query: &str) -> Option<usize> {
    let q = query.to_lowercase();
    CANON.iter().position(|b| {
        b.name.to_lowercase() == q
            || b.abbreviation.to_lowercase() == q
            || b.code.to_lowercase() == q
    })
}
