use super::*;
use crate::ui::theme::FontKind;

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fonts");

fn asset(name: &str) -> Vec<u8> {
    std::fs::read(format!("{ASSETS}/{name}")).unwrap()
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("{FIXTURES}/{name}")).unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "abcom-fonts-{name}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// En-tête WOFF2 fabriqué qui annonce `announced` octets décompressés.
fn woff_header(magic: &[u8; 4], announced: u32) -> Vec<u8> {
    let mut header = vec![0u8; 48];
    header[..4].copy_from_slice(magic);
    header[16..20].copy_from_slice(&announced.to_be_bytes());
    header
}

// ── Décodage ──────────────────────────────────────────────────────────────

#[test]
fn plain_fonts_pass_through_unchanged() {
    let ttf = asset("Inter-Regular.ttf");
    assert_eq!(decode_font(&ttf).unwrap(), ttf);
    let otf = asset("Unifont.otf");
    assert_eq!(decode_font(&otf).unwrap(), otf);
}

#[test]
fn woff_and_woff2_decode_to_a_valid_font() {
    for name in ["Inter-Regular.woff", "Inter-Regular.woff2"] {
        let decoded = decode_font(&fixture(name)).unwrap();
        assert_eq!(
            validate_font_set(&[(&decoded, 0)]).as_deref(),
            Ok("Inter"),
            "{name}"
        );
    }
}

#[test]
fn a_truncated_web_font_is_unreadable_and_does_not_panic() {
    for name in ["Inter-Regular.woff", "Inter-Regular.woff2"] {
        let raw = fixture(name);
        for cut in [4, 19, 20, 100, raw.len() / 2, raw.len() - 1] {
            let result = decode_font(&raw[..cut]);
            assert!(result.is_err(), "{name} coupé à {cut}");
        }
    }
}

#[test]
fn a_decompression_bomb_header_is_refused_before_decoding() {
    for magic in [b"wOF2", b"wOFF"] {
        let bomb = woff_header(magic, MAX_DECODED_BYTES + 1);
        assert_eq!(decode_font(&bomb), Err(FontError::TooLarge));
        let u32_max = woff_header(magic, u32::MAX);
        assert_eq!(decode_font(&u32_max), Err(FontError::TooLarge));
    }
}

#[test]
fn a_web_font_header_with_nothing_behind_it_is_unreadable() {
    assert_eq!(decode_font(b"wOF2"), Err(FontError::Unreadable));
    assert_eq!(decode_font(b"wOFF\0\0\0\0"), Err(FontError::Unreadable));
    assert!(decode_font(&woff_header(b"wOF2", 1000)).is_err());
}

// ── Validation ────────────────────────────────────────────────────────────

#[test]
fn real_fonts_validate_with_their_family_name() {
    assert_eq!(
        validate_font_set(&[(&asset("Inter-Regular.ttf"), 0)]).as_deref(),
        Ok("Inter")
    );
    assert_eq!(
        validate_font_set(&[(&asset("Unifont.otf"), 0)]).as_deref(),
        Ok("Unifont")
    );
    assert_eq!(
        validate_font_set(&[(&asset("NotoSans-Regular.ttf"), 0)]).as_deref(),
        Ok("Noto Sans")
    );
}

#[test]
fn garbage_and_degenerate_inputs_are_refused_without_panicking() {
    let junk = b"ceci n'est pas une police".to_vec();
    for bytes in [
        junk.as_slice(),
        &[],
        &[0u8],
        &[0u8, 1, 0, 0],
        &[0xFF; 64],
        b"wOF2",
    ] {
        assert!(validate_font_set(&[(bytes, 0)]).is_err(), "{bytes:?}");
    }
    assert_eq!(validate_font_set(&[]), Err(FontError::Unreadable));
}

#[test]
fn a_truncated_font_never_validates() {
    let ttf = asset("Inter-Regular.ttf");
    for cut in [0, 1, 11, 12, 100, 4000, ttf.len() / 2] {
        assert!(
            validate_font_set(&[(&ttf[..cut], 0)]).is_err(),
            "coupée à {cut}"
        );
    }
}

#[test]
fn a_collection_index_out_of_range_is_refused() {
    let ttf = asset("Inter-Regular.ttf");
    assert!(validate_font_set(&[(&ttf, 7)]).is_err());
    assert!(validate_font_set(&[(&ttf, u32::MAX)]).is_err());
}

#[test]
fn every_suggested_font_decodes_and_validates_as_a_set() {
    for font in SuggestedFont::ALL {
        let decoded = font
            .decoded()
            .unwrap_or_else(|e| panic!("{}: {e:?}", font.id()));
        let regular: Vec<(&[u8], u32)> =
            decoded.regular.iter().map(|b| (b.as_slice(), 0)).collect();
        let family = validate_font_set(&regular).unwrap_or_else(|e| panic!("{}: {e:?}", font.id()));
        assert!(!family.is_empty());
        let bold: Vec<(&[u8], u32)> = decoded.bold.iter().map(|b| (b.as_slice(), 0)).collect();
        assert!(validate_font_set(&bold).is_ok(), "{} gras", font.id());
    }
}

#[test]
fn a_latin_ext_subset_alone_is_refused_but_valid_next_to_its_latin() {
    let decoded = SuggestedFont::Lexend.decoded().unwrap();
    assert_eq!(decoded.regular.len(), 2, "latin + latin-ext");
    let (latin, ext) = (&decoded.regular[0], &decoded.regular[1]);
    assert_eq!(
        validate_font_set(&[(ext.as_slice(), 0)]),
        Err(FontError::NoLatin)
    );
    assert!(validate_font_set(&[(latin.as_slice(), 0), (ext.as_slice(), 0)]).is_ok());
    assert!(validate_font_set(&[(ext.as_slice(), 0), (latin.as_slice(), 0)]).is_ok());
}

#[test]
fn a_set_with_one_broken_file_is_refused() {
    let latin = SuggestedFont::Lexend.decoded().unwrap().regular[0].clone();
    assert_eq!(
        validate_font_set(&[(latin.as_slice(), 0), (b"pas une police", 0)]),
        Err(FontError::Unreadable)
    );
}

#[test]
fn suggested_fonts_cover_french_and_central_european_letters() {
    use skrifa::MetadataProvider;
    const FRENCH: &str = "àâçéèêëîïôûùüÿœæÀÉÇŒ€«»’";
    const EXTENDED: &str = "ąćęłńśźżőűčšžřůğış";
    for font in SuggestedFont::ALL {
        let decoded = font.decoded().unwrap();
        let faces: Vec<_> = decoded
            .regular
            .iter()
            .map(|bytes| skrifa::FontRef::new(bytes.as_slice()).unwrap())
            .collect();
        for c in FRENCH.chars().chain(EXTENDED.chars()) {
            assert!(
                faces.iter().any(|f| f.charmap().map(c).is_some()),
                "{} ne dessine pas {c}",
                font.id()
            );
        }
    }
}

#[test]
fn suggested_fonts_are_decoded_once_and_shared() {
    let a = SuggestedFont::Nunito.decoded().unwrap();
    let b = SuggestedFont::Nunito.decoded().unwrap();
    assert!(Arc::ptr_eq(&a.regular[0], &b.regular[0]));
}

#[test]
fn suggested_fonts_have_distinct_ids_and_names() {
    let ids: std::collections::HashSet<_> = SuggestedFont::ALL.iter().map(|f| f.id()).collect();
    let names: std::collections::HashSet<_> = SuggestedFont::ALL.iter().map(|f| f.name()).collect();
    assert_eq!((ids.len(), names.len()), (6, 6));
    for font in SuggestedFont::ALL {
        assert_eq!(SuggestedFont::from_id(font.id()), Some(font));
    }
    assert_eq!(SuggestedFont::from_id("comic-sans"), None);
}

// ── Choix ─────────────────────────────────────────────────────────────────

#[test]
fn font_choices_round_trip_through_their_id() {
    let imported = FontChoice::Imported("f1a2b".into());
    for choice in [
        FontChoice::ThemeDefault,
        FontChoice::NotoSans,
        FontChoice::Inter,
        FontChoice::Suggested(SuggestedFont::OpenDyslexic),
        imported,
    ] {
        assert_eq!(FontChoice::from_id(&choice.to_id()), choice);
        let json = serde_json::to_string(&choice).unwrap();
        assert_eq!(serde_json::from_str::<FontChoice>(&json).unwrap(), choice);
    }
}

#[test]
fn an_unknown_or_hostile_choice_falls_back_to_the_theme_font() {
    for id in [
        "",
        "inconnu",
        "suggested:",
        "suggested:comic-sans",
        "imported:",
        "imported:../etc/passwd",
        "imported:x1",
        "imported:FABC",
        "imported:f/../x",
        "imported: f1",
        "🙂",
    ] {
        assert_eq!(FontChoice::from_id(id), FontChoice::ThemeDefault, "{id:?}");
    }
    assert_eq!(
        serde_json::from_str::<FontChoice>("\"n'importe quoi\"").unwrap(),
        FontChoice::ThemeDefault
    );
    assert!(serde_json::from_str::<FontChoice>("42").is_err());
}

// ── Magasin ───────────────────────────────────────────────────────────────

#[test]
fn importing_each_format_copies_a_decoded_ttf_and_names_the_family() {
    for (name, source) in [
        ("ttf", asset("Inter-Regular.ttf")),
        ("woff", fixture("Inter-Regular.woff")),
        ("woff2", fixture("Inter-Regular.woff2")),
    ] {
        let dir = temp_dir(&format!("import-{name}"));
        let file = write(&dir, &format!("source.{name}"), &source);
        let mut store = FontStore::load(&dir.join("fonts"));
        let outcome = store
            .import(&file)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let ImportOutcome::Added(font) = outcome else {
            panic!("{name} : ajout attendu")
        };
        assert_eq!(font.family, "Inter");
        assert!(valid_font_id(&font.id));
        let copy = std::fs::read(dir.join("fonts").join(&font.file)).unwrap();
        assert_eq!(
            validate_font_set(&[(&copy, 0)]).as_deref(),
            Ok("Inter"),
            "toujours un TTF"
        );
        assert!(!copy.starts_with(b"wOF"), "{name} : la copie est décodée");
        assert_eq!(store.read(&font.id).unwrap().as_slice(), copy.as_slice());
        assert!(!dir.join("fonts").join(format!("{}.part", font.id)).exists());
    }
}

#[test]
fn imported_fonts_survive_a_reload() {
    let dir = temp_dir("reload");
    let source = write(&dir, "a.ttf", &asset("Inter-Regular.ttf"));
    let mut store = FontStore::load(&dir.join("fonts"));
    let ImportOutcome::Added(font) = store.import(&source).unwrap() else {
        unreachable!()
    };
    let reloaded = FontStore::load(&dir.join("fonts"));
    assert_eq!(reloaded.fonts(), std::slice::from_ref(&font));
    assert!(reloaded.read(&font.id).is_ok());
}

#[test]
fn importing_the_same_content_twice_selects_the_existing_font() {
    let dir = temp_dir("dedup");
    let mut store = FontStore::load(&dir.join("fonts"));
    let first = write(&dir, "a.ttf", &asset("Inter-Regular.ttf"));
    let renamed = write(
        &dir,
        "completement autre nom é🙂.ttf",
        &asset("Inter-Regular.ttf"),
    );
    let ImportOutcome::Added(added) = store.import(&first).unwrap() else {
        unreachable!()
    };
    let ImportOutcome::Existing(existing) = store.import(&renamed).unwrap() else {
        panic!("doublon non détecté")
    };
    assert_eq!(existing.id, added.id);
    assert_eq!(store.fonts().len(), 1);
    let files = std::fs::read_dir(dir.join("fonts")).unwrap().count();
    assert_eq!(files, 2, "un TTF et l'index, pas de doublon");
}

#[test]
fn two_different_fonts_with_the_same_family_name_are_both_kept_and_numbered() {
    let dir = temp_dir("same-family");
    let mut store = FontStore::load(&dir.join("fonts"));
    store.import_bytes(&asset("Inter-Regular.ttf")).unwrap();
    store.import_bytes(&asset("Inter-Italic.ttf")).unwrap();
    store.import_bytes(&asset("Inter-SemiBold.ttf")).unwrap();
    let families: Vec<_> = store.fonts().iter().map(|f| f.family.clone()).collect();
    let unique: std::collections::HashSet<_> = families.iter().collect();
    assert_eq!(unique.len(), 3, "{families:?}");
    assert_eq!(store.fonts().len(), 3);
}

#[test]
fn refused_imports_leave_nothing_behind() {
    let dir = temp_dir("refused");
    let fonts_dir = dir.join("fonts");
    let mut store = FontStore::load(&fonts_dir);
    let ttf = asset("Inter-Regular.ttf");
    let woff2 = fixture("Inter-Regular.woff2");
    let cases: Vec<(&str, Vec<u8>, FontError)> = vec![
        (
            "bidon",
            b"ceci n'est pas une police".to_vec(),
            FontError::Unreadable,
        ),
        ("vide", Vec::new(), FontError::Unreadable),
        ("tronquée", ttf[..4000].to_vec(), FontError::NoLatin),
        (
            "woff2 tronqué",
            woff2[..woff2.len() / 2].to_vec(),
            FontError::Unreadable,
        ),
        ("bombe", woff_header(b"wOF2", u32::MAX), FontError::TooLarge),
        (
            "sous-ensemble latin-ext seul",
            SuggestedFont::Lexend.decoded().unwrap().regular[1]
                .as_ref()
                .clone(),
            FontError::NoLatin,
        ),
    ];
    for (name, bytes, _) in &cases {
        let file = write(&dir, "x.ttf", bytes);
        let result = store.import(&file);
        assert!(
            matches!(result, Err(ImportError::Font(_))),
            "{name} : {result:?}"
        );
    }
    assert!(store.fonts().is_empty());
    assert!(!fonts_dir.exists() || std::fs::read_dir(&fonts_dir).unwrap().next().is_none());
    assert_eq!(store.revision, 0);
}

#[test]
fn an_oversized_file_is_refused_without_being_read() {
    let dir = temp_dir("huge");
    let mut store = FontStore::load(&dir.join("fonts"));
    let file = dir.join("enorme.ttf");
    let handle = std::fs::File::create(&file).unwrap();
    handle.set_len(MAX_FONT_FILE_BYTES + 1).unwrap();
    assert_eq!(
        store.import(&file),
        Err(ImportError::Font(FontError::TooLarge))
    );
    assert_eq!(
        store.import_bytes(&vec![0u8; (MAX_FONT_FILE_BYTES + 1) as usize]),
        Err(ImportError::Font(FontError::TooLarge))
    );
}

#[test]
fn a_missing_source_is_an_io_error() {
    let dir = temp_dir("missing");
    let mut store = FontStore::load(&dir.join("fonts"));
    assert!(matches!(
        store.import(&dir.join("absent.ttf")),
        Err(ImportError::Io(_))
    ));
    assert!(
        matches!(store.import(&dir), Err(ImportError::Io(_))),
        "un dossier n'est pas une police"
    );
}

#[test]
fn the_displayed_name_comes_from_the_font_not_from_the_file_name() {
    let dir = temp_dir("naming");
    let hostile = write(
        &dir,
        "../../etc é 🙂 passwd.ttf".replace('/', "_").as_str(),
        &asset("Inter-Regular.ttf"),
    );
    let mut store = FontStore::load(&dir.join("fonts"));
    let ImportOutcome::Added(font) = store.import(&hostile).unwrap() else {
        unreachable!()
    };
    assert_eq!(font.family, "Inter");
    assert!(valid_font_id(&font.id));
    assert!(dir.join("fonts").join(&font.file).is_file());
}

#[test]
fn remove_deletes_the_file_and_the_index_entry() {
    let dir = temp_dir("remove");
    let mut store = FontStore::load(&dir.join("fonts"));
    let ImportOutcome::Added(font) = store.import_bytes(&asset("Inter-Regular.ttf")).unwrap()
    else {
        unreachable!()
    };
    let revision = store.revision;
    assert!(store.remove(&font.id));
    assert!(store.revision > revision);
    assert!(!dir.join("fonts").join(&font.file).exists());
    assert!(FontStore::load(&dir.join("fonts")).fonts().is_empty());
    assert!(!store.remove(&font.id), "déjà retirée");
    assert!(!store.remove("f0"));
}

#[test]
fn loading_drops_entries_whose_file_is_missing_or_hostile() {
    let dir = temp_dir("load-bad");
    let fonts_dir = dir.join("fonts");
    let mut store = FontStore::load(&fonts_dir);
    let ImportOutcome::Added(kept) = store.import_bytes(&asset("Inter-Regular.ttf")).unwrap()
    else {
        unreachable!()
    };
    let ImportOutcome::Added(gone) = store.import_bytes(&asset("Inter-Italic.ttf")).unwrap() else {
        unreachable!()
    };
    std::fs::remove_file(fonts_dir.join(&gone.file)).unwrap();
    let mut index: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(fonts_dir.join("index.json")).unwrap())
            .unwrap();
    index.push(serde_json::json!({"id":"../x","family":"x","hash":"h","file":"../x.ttf"}));
    index.push(serde_json::json!({"id":"f9","family":"y","hash":"h","file":"autre.ttf"}));
    std::fs::write(
        fonts_dir.join("index.json"),
        serde_json::to_string(&index).unwrap(),
    )
    .unwrap();

    let loaded = FontStore::load(&fonts_dir);
    assert_eq!(loaded.fonts().len(), 1);
    assert_eq!(loaded.fonts()[0].id, kept.id);
}

#[test]
fn a_corrupt_or_missing_index_gives_an_empty_store() {
    let dir = temp_dir("index");
    assert!(FontStore::load(&dir.join("absent")).fonts().is_empty());
    for junk in ["", "{", "null", "42", "[1,2]", "[{\"id\":3}]"] {
        write(&dir, "index.json", junk.as_bytes());
        assert!(FontStore::load(&dir).fonts().is_empty(), "{junk:?}");
    }
}

/// Un fichier remplacé à la main par n'importe quoi ne doit pas atteindre egui.
#[test]
fn reading_a_tampered_font_file_is_refused_instead_of_crashing() {
    let dir = temp_dir("tamper");
    let fonts_dir = dir.join("fonts");
    let mut store = FontStore::load(&fonts_dir);
    let ImportOutcome::Added(font) = store.import_bytes(&asset("Inter-Regular.ttf")).unwrap()
    else {
        unreachable!()
    };
    std::fs::write(fonts_dir.join(&font.file), b"remplace par du texte").unwrap();
    assert_eq!(store.read(&font.id), Err(FontError::Unreadable));
    std::fs::write(fonts_dir.join(&font.file), b"").unwrap();
    assert!(store.read(&font.id).is_err());
    std::fs::remove_file(fonts_dir.join(&font.file)).unwrap();
    assert!(store.read(&font.id).is_err());
    assert!(store.read("inconnu").is_err());
}

#[cfg(unix)]
#[test]
fn an_unwritable_directory_reports_an_io_error_and_leaves_no_entry() {
    use std::os::unix::fs::PermissionsExt;
    let dir = temp_dir("readonly");
    let fonts_dir = dir.join("fonts");
    std::fs::create_dir_all(&fonts_dir).unwrap();
    std::fs::set_permissions(&fonts_dir, std::fs::Permissions::from_mode(0o500)).unwrap();
    if std::fs::write(fonts_dir.join("sonde"), "x").is_ok() {
        std::fs::set_permissions(&fonts_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        return;
    }
    let mut store = FontStore::load(&fonts_dir);
    let result = store.import_bytes(&asset("Inter-Regular.ttf"));
    std::fs::set_permissions(&fonts_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(result, Err(ImportError::Io(_))), "{result:?}");
    assert!(store.fonts().is_empty());
}

// ── Plans ─────────────────────────────────────────────────────────────────

#[test]
fn plans_report_their_real_cuts() {
    assert!(FontPlan::Theme(FontKind::Inter).real_bold());
    assert!(FontPlan::Theme(FontKind::Inter).real_italic());
    assert!(!FontPlan::Theme(FontKind::NotoSans).real_bold());
    assert!(!FontPlan::Theme(FontKind::NotoSans).real_italic());
    let suggested = resolve_choice(
        &FontChoice::Suggested(SuggestedFont::Lexend),
        FontKind::NotoSans,
        &FontStore::load(&temp_dir("plan")),
    )
    .unwrap();
    assert!(
        suggested.real_bold(),
        "les polices proposées embarquent leur gras"
    );
    assert!(!suggested.real_italic());
}

#[test]
fn an_imported_font_has_no_real_bold_and_its_key_follows_its_content() {
    let dir = temp_dir("plan-imported");
    let mut store = FontStore::load(&dir);
    let ImportOutcome::Added(font) = store.import_bytes(&asset("Inter-Regular.ttf")).unwrap()
    else {
        unreachable!()
    };
    let plan = resolve_choice(
        &FontChoice::Imported(font.id.clone()),
        FontKind::NotoSans,
        &store,
    )
    .unwrap();
    assert!(!plan.real_bold());
    assert!(plan.key().contains(&font.hash));
}

#[test]
fn a_missing_imported_font_cannot_be_resolved() {
    let store = FontStore::load(&temp_dir("plan-missing"));
    assert_eq!(
        resolve_choice(&FontChoice::Imported("f1".into()), FontKind::Inter, &store),
        Err(FontError::Unreadable)
    );
}

#[test]
fn theme_default_follows_the_theme_font() {
    let store = FontStore::load(&temp_dir("plan-theme"));
    for kind in [FontKind::NotoSans, FontKind::Inter] {
        assert_eq!(
            resolve_choice(&FontChoice::ThemeDefault, kind, &store),
            Ok(FontPlan::Theme(kind))
        );
    }
    assert_eq!(
        resolve_choice(&FontChoice::Inter, FontKind::NotoSans, &store),
        Ok(FontPlan::Theme(FontKind::Inter))
    );
}
