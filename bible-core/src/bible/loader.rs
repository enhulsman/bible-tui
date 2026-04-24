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

pub fn load_from_bytes(data: &[u8]) -> Result<BibleData, postcard::Error> {
    postcard::from_bytes(data)
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
    fn load_web() {
        let bible = load_bundled(BundledTranslation::Web);
        assert_eq!(bible.books.len(), 66);
    }

    #[test]
    fn load_sv() {
        let bible = load_bundled(BundledTranslation::Sv);
        assert!(bible.books.len() >= 60);
    }
}
