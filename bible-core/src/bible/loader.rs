use super::model::{BibleData, BundledTranslation};

const KJV_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kjv.postcard"));

// WEB and SV are excluded from the WASM binary to keep bundle size small.
// In the web build, they are fetched on demand from /translations/*.postcard.
#[cfg(not(feature = "wasm"))]
const WEB_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/web.postcard"));
#[cfg(not(feature = "wasm"))]
const SV_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sv.postcard"));

pub fn load_bundled(translation: BundledTranslation) -> BibleData {
    #[cfg(feature = "wasm")]
    {
        match translation {
            BundledTranslation::Kjv => postcard::from_bytes(KJV_DATA)
                .unwrap_or_else(|e| panic!("Failed to deserialize KJV: {e}")),
            // On WASM, WEB/SV must be fetched asynchronously — this path should not be called
            _ => panic!("Translation {:?} must be fetched asynchronously in WASM builds", translation),
        }
    }
    #[cfg(not(feature = "wasm"))]
    {
        let data = match translation {
            BundledTranslation::Kjv => KJV_DATA,
            BundledTranslation::Web => WEB_DATA,
            BundledTranslation::Sv => SV_DATA,
        };
        postcard::from_bytes(data)
            .unwrap_or_else(|e| panic!("Failed to deserialize {:?}: {e}", translation))
    }
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

    #[cfg(not(feature = "wasm"))]
    #[test]
    fn load_web() {
        let bible = load_bundled(BundledTranslation::Web);
        assert_eq!(bible.books.len(), 66);
    }

    #[cfg(not(feature = "wasm"))]
    #[test]
    fn load_sv() {
        let bible = load_bundled(BundledTranslation::Sv);
        assert!(bible.books.len() >= 60);
    }
}
