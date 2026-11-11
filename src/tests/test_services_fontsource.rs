use super::*;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fontsource/fonts.json"
);

fn fixture() -> Vec<u8> {
    std::fs::read(FIXTURE).unwrap()
}

fn catalog() -> Vec<CatalogFont> {
    parse_catalog(&fixture()).unwrap()
}

fn font(id: &str, family: &str) -> CatalogFont {
    CatalogFont {
        id: id.into(),
        family: family.into(),
        subsets: vec!["latin".into(), "latin-ext".into()],
        weights: vec![400, 700],
        styles: vec!["normal".into()],
        category: "sans-serif".into(),
        license: "OFL-1.1".into(),
    }
}

#[test]
fn the_real_catalog_extract_keeps_only_free_licenses() {
    let fonts = catalog();
    let ids: Vec<_> = fonts.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(fonts.len(), 8, "{ids:?}");
    assert!(!ids.contains(&"aileron"), "licence CC0 : écartée");
    for id in [
        "inter",
        "lexend",
        "atkinson-hyperlegible",
        "jetbrains-mono",
        "noto-sans-jp",
    ] {
        assert!(ids.contains(&id), "{id}");
    }
    assert!(fonts.iter().all(|f| LICENSES.contains(&f.license.as_str())));
}

#[test]
fn the_catalog_is_sorted_by_name() {
    let names: Vec<_> = catalog().iter().map(|f| fold(&f.family)).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn malformed_lists_are_refused_without_panicking() {
    for raw in [
        &b""[..],
        b"{",
        b"null",
        b"42",
        b"{}",
        b"[]",
        b"[1,2,3]",
        b"[{\"id\":3}]",
        b"\xff\xfe",
        b"<html>erreur</html>",
    ] {
        assert!(
            parse_catalog(raw).is_err(),
            "{:?}",
            String::from_utf8_lossy(raw)
        );
    }
    assert!(parse_catalog(&vec![b' '; MAX_LIST_BYTES + 1]).is_err());
}

#[test]
fn entries_with_missing_or_hostile_fields_are_skipped_not_fatal() {
    let raw = serde_json::json!([
        {"id":"bonne","family":"Bonne","subsets":["latin"],"weights":[400],"license":"OFL-1.1"},
        {"id":"../etc","family":"Piège","subsets":["latin"],"license":"OFL-1.1"},
        {"id":"Majuscule","family":"X","subsets":["latin"],"license":"OFL-1.1"},
        {"id":"sans-licence","family":"Y","subsets":["latin"]},
        {"id":"mit","family":"Z","subsets":["latin"],"license":"MIT"},
        {"id":"sans-latin","family":"W","subsets":["cyrillic"],"license":"OFL-1.1"},
        {"id":"","family":"V","subsets":["latin"],"license":"OFL-1.1"},
        {"id":"vide","family":"  ","subsets":["latin"],"license":"OFL-1.1"},
        {"id":"bonne","family":"Doublon","subsets":["latin"],"license":"OFL-1.1"},
        {"id":"apache","family":"Apache","subsets":["latin"],"license":"Apache-2.0"},
        "pas un objet",
        {"id":"typé-faux","family":3,"subsets":["latin"],"license":"OFL-1.1"}
    ])
    .to_string();
    let fonts = parse_catalog(raw.as_bytes()).unwrap();
    let ids: Vec<_> = fonts.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(ids, ["apache", "bonne"]);
}

#[test]
fn ids_are_validated() {
    for ok in ["inter", "jetbrains-mono", "noto-sans-jp", "42dot-sans"] {
        assert!(valid_font_id(ok), "{ok}");
    }
    for bad in [
        "",
        "Inter",
        "../x",
        "a/b",
        "a b",
        "é",
        "a.b",
        "a_b",
        &"a".repeat(81),
    ] {
        assert!(!valid_font_id(bad), "{bad:?}");
    }
}

#[test]
fn search_ignores_case_and_accents() {
    let fonts = vec![
        font("lexend", "Lexend"),
        font("fraunces", "Fraunces"),
        font("cafe", "Café Étoilé"),
    ];
    let names = |q: &str, c: Option<&str>| -> Vec<String> {
        filter(&fonts, q, c)
            .iter()
            .map(|f| f.family.clone())
            .collect()
    };
    assert_eq!(names("lex", None), ["Lexend"]);
    assert_eq!(names("LEX", None), ["Lexend"]);
    assert_eq!(names("  lex  ", None), ["Lexend"]);
    assert_eq!(names("cafe etoile", None), ["Café Étoilé"]);
    assert_eq!(names("CAFÉ", None), ["Café Étoilé"]);
    assert_eq!(names("", None).len(), 3);
    assert!(names("zzz", None).is_empty());
    assert!(names("lex", Some("serif")).is_empty());
    assert_eq!(names("lex", Some("sans-serif")), ["Lexend"]);
}

#[test]
fn search_inputs_that_look_like_patterns_are_plain_text() {
    let fonts = vec![font("lexend", "Lexend")];
    for q in [".*", "(", "[", "\\", "%", "lex end", "🙂", "\u{0}"] {
        assert!(filter(&fonts, q, None).is_empty(), "{q:?}");
    }
}

#[test]
fn pagination_never_overflows() {
    let items: Vec<usize> = (0..120).collect();
    assert_eq!(page_count(0), 1);
    assert_eq!(page_count(1), 1);
    assert_eq!(page_count(50), 1);
    assert_eq!(page_count(51), 2);
    assert_eq!(page_count(120), 3);
    assert_eq!(page_of(&items, 0).len(), 50);
    assert_eq!(page_of(&items, 2).len(), 20);
    assert_eq!(
        page_of(&items, 99),
        page_of(&items, 2),
        "page hors bornes = la dernière"
    );
    assert!(page_of::<usize>(&[], 0).is_empty());
    assert!(page_of::<usize>(&[], 5).is_empty());
}

#[test]
fn urls_point_to_the_cdn_with_a_safe_path() {
    assert_eq!(
        file_url("inter", "latin", 400),
        "https://cdn.jsdelivr.net/fontsource/fonts/inter@latest/latin-400-normal.woff2"
    );
}

#[test]
fn a_full_font_fetches_latin_latin_ext_and_the_bold_cuts() {
    let files = files_to_fetch(&font("inter", "Inter"));
    let parts: Vec<_> = files.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        parts,
        [
            Part::Latin,
            Part::LatinExt,
            Part::BoldLatin,
            Part::BoldLatinExt
        ]
    );
    assert!(files[3].1.ends_with("latin-ext-700-normal.woff2"));
}

#[test]
fn missing_subsets_and_weights_are_not_requested() {
    let mut only_latin = font("a", "A");
    only_latin.subsets = vec!["latin".into()];
    only_latin.weights = vec![400];
    assert_eq!(files_to_fetch(&only_latin).len(), 1);

    let mut light_only = font("b", "B");
    light_only.weights = vec![300];
    let files = files_to_fetch(&light_only);
    assert!(
        files[0].1.contains("-300-"),
        "poids le plus proche : {}",
        files[0].1
    );
    assert_eq!(files.len(), 2, "pas de gras");

    let mut no_weights = font("c", "C");
    no_weights.weights = vec![];
    assert!(files_to_fetch(&no_weights)[0].1.contains("-400-"));

    let mut bold_only = font("d", "D");
    bold_only.weights = vec![700];
    assert!(
        files_to_fetch(&bold_only).iter().all(|(p, _)| !p.is_bold()),
        "700 seul : c'est la coupe normale"
    );
}

#[test]
fn only_the_latin_file_is_required() {
    assert!(Part::Latin.is_required());
    for part in [Part::LatinExt, Part::BoldLatin, Part::BoldLatinExt] {
        assert!(!part.is_required());
    }
}

#[test]
fn the_cache_round_trips_and_reports_its_age() {
    let dir = std::env::temp_dir().join(format!("abcom-fs-{}", now_secs()));
    let path = dir.join("fontsource.json");
    write_cache(&path, &catalog(), 1_000);
    let (fonts, age) = read_cache(&path, 1_500).unwrap();
    assert_eq!(fonts, catalog());
    assert_eq!(age, 500);
    assert!(!dir.join("fontsource.part").exists());
    assert_eq!(
        read_cache(&path, 500).unwrap().1,
        0,
        "horloge reculée : âge nul"
    );
}

#[test]
fn a_damaged_or_hostile_cache_is_ignored() {
    let dir = std::env::temp_dir().join(format!("abcom-fs-bad-{}", now_secs()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fontsource.json");
    for junk in [
        "",
        "{",
        "null",
        "[]",
        "{\"fetched_at\":1}",
        "{\"fetched_at\":1,\"fonts\":[]}",
    ] {
        std::fs::write(&path, junk).unwrap();
        assert!(read_cache(&path, 10).is_none(), "{junk:?}");
    }
    let hostile = serde_json::json!({"fetched_at":1,"fonts":[
        {"id":"../x","family":"x","license":"OFL-1.1"},
        {"id":"mit","family":"y","license":"MIT"}
    ]});
    std::fs::write(&path, hostile.to_string()).unwrap();
    assert!(read_cache(&path, 10).is_none());
    assert!(read_cache(&dir.join("absent.json"), 10).is_none());
}

#[test]
fn writing_the_cache_into_an_impossible_place_is_silent() {
    write_cache(
        Path::new("/proc/nexiste/pas/fontsource.json"),
        &catalog(),
        1,
    );
}
