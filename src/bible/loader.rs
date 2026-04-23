use super::model::{BibleData, BundledTranslation};

const KJV_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kjv.postcard"));
const WEB_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/web.postcard"));
const SV_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sv.postcard"));

pub fn load_bundled(translation: BundledTranslation) -> BibleData {
    let data = match translation {
        BundledTranslation::Kjv => KJV_DATA,
        BundledTranslation::Web => WEB_DATA,
        BundledTranslation::Sv => SV_DATA,
    };
    postcard::from_bytes(data)
        .unwrap_or_else(|e| panic!("Failed to deserialize {:?}: {e}", translation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_kjv() {
        let bible = load_bundled(BundledTranslation::Kjv);
        assert_eq!(bible.books.len(), 66);
    }

    #[test]
    fn genesis_1_1() {
        let bible = load_bundled(BundledTranslation::Kjv);
        let genesis = &bible.books[0];
        assert_eq!(genesis.name, "Genesis");
        let ch1 = &genesis.chapters[0];
        assert_eq!(ch1.number, 1);
        let v1 = &ch1.verses[0];
        assert_eq!(v1.number, 1);
        assert!(v1.text().starts_with("In the beginning God created"));
    }

    #[test]
    fn john_3_16() {
        let bible = load_bundled(BundledTranslation::Kjv);
        let john = &bible.books[42];
        assert_eq!(john.name, "John");
        let ch3 = &john.chapters[2];
        assert_eq!(ch3.number, 3);
        let v16 = &ch3.verses[15];
        assert_eq!(v16.number, 16);
        assert!(v16.text().contains("For God so loved the world"));
    }

    #[test]
    fn revelation_22_21() {
        let bible = load_bundled(BundledTranslation::Kjv);
        let rev = &bible.books[65];
        assert_eq!(rev.name, "Revelation");
        let last_ch = rev.chapters.last().unwrap();
        assert_eq!(last_ch.number, 22);
        let last_v = last_ch.verses.last().unwrap();
        assert_eq!(last_v.number, 21);
        assert!(last_v.text().contains("grace of our Lord Jesus Christ"));
    }

    #[test]
    fn load_web() {
        let bible = load_bundled(BundledTranslation::Web);
        assert_eq!(bible.books.len(), 66);
        let gen = &bible.books[0];
        assert_eq!(gen.name, "Genesis");
        let v1 = &gen.chapters[0].verses[0];
        assert!(v1.text().contains("In the beginning"));
    }

    #[test]
    fn web_red_letter() {
        use crate::bible::model::VerseSpan;
        let bible = load_bundled(BundledTranslation::Web);
        // John 3:16 in WEB should have red-letter text (words of Jesus)
        let john = &bible.books[42];
        let ch3 = &john.chapters[2];
        let v16 = &ch3.verses[15];
        let has_red = v16
            .spans
            .iter()
            .any(|s| matches!(s, VerseSpan::RedLetter(_)));
        assert!(has_red, "John 3:16 should contain red-letter text in WEB");
    }

    #[test]
    fn load_sv() {
        let bible = load_bundled(BundledTranslation::Sv);
        assert!(bible.books.len() >= 60, "SV should have at least 60 books, got {}", bible.books.len());
        let gen = &bible.books[0];
        assert_eq!(gen.name, "Genesis");
        let v1 = &gen.chapters[0].verses[0];
        assert!(v1.text().contains("beginne"), "Gen 1:1 SV should contain 'beginne': {}", v1.text());
    }
}
