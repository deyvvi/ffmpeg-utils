use std::collections::HashMap;

pub fn normalize_language(raw: &str) -> String {
    let value = raw.trim().to_lowercase().replace('_', "-");
    let aliases: HashMap<&str, &str> = HashMap::from([
        ("en", "eng"),
        ("eng", "eng"),
        ("english", "eng"),
        ("en-us", "eng"),
        ("en-gb", "eng"),
        ("es", "spa"),
        ("spa", "spa"),
        ("spanish", "spa"),
        ("es-es", "spa"),
        ("es-mx", "spa"),
        ("es-419", "spa"),
        ("es-la", "spa"),
        ("latino", "spa"),
        ("latin", "spa"),
        ("lat", "spa"),
        ("fr", "fra"),
        ("fre", "fra"),
        ("fra", "fra"),
        ("french", "fra"),
        ("de", "deu"),
        ("ger", "deu"),
        ("deu", "deu"),
        ("german", "deu"),
        ("it", "ita"),
        ("ita", "ita"),
        ("italian", "ita"),
        ("pt", "por"),
        ("por", "por"),
        ("portuguese", "por"),
        ("ja", "jpn"),
        ("jpn", "jpn"),
        ("japanese", "jpn"),
        ("ko", "kor"),
        ("kor", "kor"),
        ("korean", "kor"),
        ("zh", "zho"),
        ("chi", "zho"),
        ("zho", "zho"),
        ("chinese", "zho"),
        ("ru", "rus"),
        ("rus", "rus"),
        ("russian", "rus"),
        ("ar", "ara"),
        ("ara", "ara"),
        ("arabic", "ara"),
        ("hi", "hin"),
        ("hin", "hin"),
        ("hindi", "hin"),
        ("tr", "tur"),
        ("tur", "tur"),
        ("turkish", "tur"),
        ("pl", "pol"),
        ("pol", "pol"),
        ("polish", "pol"),
        ("nl", "nld"),
        ("dut", "nld"),
        ("nld", "nld"),
        ("dutch", "nld"),
        ("sv", "swe"),
        ("swe", "swe"),
        ("swedish", "swe"),
        ("no", "nor"),
        ("nor", "nor"),
        ("norwegian", "nor"),
        ("da", "dan"),
        ("dan", "dan"),
        ("danish", "dan"),
        ("fi", "fin"),
        ("fin", "fin"),
        ("finnish", "fin"),
        ("cs", "ces"),
        ("cze", "ces"),
        ("ces", "ces"),
        ("czech", "ces"),
        ("hu", "hun"),
        ("hun", "hun"),
        ("hungarian", "hun"),
        ("el", "ell"),
        ("gre", "ell"),
        ("ell", "ell"),
        ("greek", "ell"),
        ("he", "heb"),
        ("heb", "heb"),
        ("hebrew", "heb"),
        ("uk", "ukr"),
        ("ukr", "ukr"),
        ("ukrainian", "ukr"),
        ("th", "tha"),
        ("tha", "tha"),
        ("thai", "tha"),
        ("vi", "vie"),
        ("vie", "vie"),
        ("vietnamese", "vie"),
        ("id", "ind"),
        ("ind", "ind"),
        ("indonesian", "ind"),
        ("ms", "msa"),
        ("may", "msa"),
        ("msa", "msa"),
        ("malay", "msa"),
        ("ro", "ron"),
        ("rum", "ron"),
        ("ron", "ron"),
        ("romanian", "ron"),
        ("ca", "cat"),
        ("cat", "cat"),
        ("catalan", "cat"),
        ("eu", "eus"),
        ("baq", "eus"),
        ("eus", "eus"),
        ("basque", "eus"),
        ("gl", "glg"),
        ("glg", "glg"),
        ("galician", "glg"),
        ("und", "und"),
        ("unknown", "und"),
        ("unk", "und"),
        ("mul", "und"),
        ("zxx", "und"),
        ("", "und"),
    ]);

    if let Some(value) = aliases.get(value.as_str()) {
        return (*value).to_string();
    }
    let base = value.split('-').next().unwrap_or("");
    if let Some(value) = aliases.get(base) {
        return (*value).to_string();
    }
    if (base.len() == 2 || base.len() == 3) && base.chars().all(char::is_alphabetic) {
        return base.to_string();
    }
    "und".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_codes() {
        assert_eq!(normalize_language("EN"), "eng");
        assert_eq!(normalize_language("es-419"), "spa");
        assert_eq!(normalize_language("latino"), "spa");
        assert_eq!(normalize_language("unknown"), "und");
        assert_eq!(normalize_language("xyz"), "xyz");
    }
}
