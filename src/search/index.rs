use std::collections::HashMap;

use crate::bible::model::{BibleData, VerseRef};

pub struct SearchResult {
    pub verse_ref: VerseRef,
    pub text: String,
}

pub struct SearchIndex {
    /// word → list of verse refs containing that word
    index: HashMap<String, Vec<VerseRef>>,
    /// All verse texts for snippet extraction
    verses: Vec<(VerseRef, String)>,
}

impl SearchIndex {
    pub fn build(bible: &BibleData) -> Self {
        let mut index: HashMap<String, Vec<VerseRef>> = HashMap::new();
        let mut verses = Vec::new();

        for book in &bible.books {
            for chapter in &book.chapters {
                for verse in &chapter.verses {
                    let vref = VerseRef {
                        book_index: book.index,
                        chapter: chapter.number,
                        verse: verse.number,
                    };
                    let text = verse.text();
                    let words = tokenize(&text);
                    for word in words {
                        index.entry(word).or_default().push(vref);
                    }
                    verses.push((vref, text));
                }
            }
        }

        Self { index, verses }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_words = tokenize(query);
        if query_words.is_empty() {
            return vec![];
        }

        // Get posting lists for all query words
        let mut posting_lists: Vec<&Vec<VerseRef>> = Vec::new();
        for word in &query_words {
            if let Some(list) = self.index.get(word) {
                posting_lists.push(list);
            } else {
                // If any word has no matches, result is empty
                return vec![];
            }
        }

        // Intersect posting lists
        let mut result_refs: Vec<VerseRef> = posting_lists[0].clone();
        for list in &posting_lists[1..] {
            let set: std::collections::HashSet<VerseRef> = list.iter().copied().collect();
            result_refs.retain(|vr| set.contains(vr));
        }

        // Build results with text
        result_refs
            .into_iter()
            .filter_map(|vref| {
                let text = self
                    .verses
                    .iter()
                    .find(|(vr, _)| *vr == vref)
                    .map(|(_, t)| t.clone())?;
                Some(SearchResult {
                    verse_ref: vref,
                    text,
                })
            })
            .collect()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bible::loader;
    use crate::bible::model::BundledTranslation;

    #[test]
    fn search_finds_god_so_loved() {
        let bible = loader::load_bundled(BundledTranslation::Kjv);
        let index = SearchIndex::build(&bible);
        let results = index.search("God so loved the world");
        assert!(!results.is_empty());
        // John 3:16 should be in results
        assert!(results
            .iter()
            .any(|r| r.verse_ref.book_index == 42
                && r.verse_ref.chapter == 3
                && r.verse_ref.verse == 16));
    }

    #[test]
    fn search_finds_in_the_beginning() {
        let bible = loader::load_bundled(BundledTranslation::Kjv);
        let index = SearchIndex::build(&bible);
        let results = index.search("In the beginning God created");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.verse_ref.book_index == 0
                && r.verse_ref.chapter == 1
                && r.verse_ref.verse == 1));
    }

    #[test]
    fn search_empty_query() {
        let bible = loader::load_bundled(BundledTranslation::Kjv);
        let index = SearchIndex::build(&bible);
        let results = index.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn search_nonexistent_word() {
        let bible = loader::load_bundled(BundledTranslation::Kjv);
        let index = SearchIndex::build(&bible);
        let results = index.search("xyzzyplugh");
        assert!(results.is_empty());
    }
}
