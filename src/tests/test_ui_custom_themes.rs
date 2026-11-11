use super::*;
use crate::ui::theme::{DARK, LIGHT};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "abcom-themes-{name}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn frost() -> ResolvedTheme {
    BuiltinTheme::Frost.resolve()
}

fn store_with_one(name: &str) -> (ThemeStore, String, PathBuf) {
    let dir = temp_dir(name);
    let mut store = ThemeStore::load(&dir);
    let id = store.create(BuiltinTheme::Frost, &frost(), "Mon thème");
    (store, id, dir)
}

#[test]
fn palette_data_round_trips_the_builtin_palettes() {
    for palette in [DARK, LIGHT, frost().dark, frost().light] {
        let back: Palette = PaletteData::from(palette).into();
        assert_eq!(back, palette);
    }
}

#[test]
fn a_translucent_tint_survives_the_round_trip_up_to_rounding() {
    let data = PaletteData::from(DARK);
    assert_eq!(data.row_tint, [255, 255, 255, 7]);
    let twice: PaletteData = Palette::from(data).into();
    assert_eq!(twice, data, "la seconde passe ne dérive plus");
}

#[test]
fn names_are_cleaned() {
    assert_eq!(clean_name("  Mon   thème \n"), Ok("Mon thème".into()));
    assert_eq!(clean_name("a\tb\u{0}c"), Ok("a b c".into()));
    assert_eq!(clean_name(""), Err(NameError::Empty));
    assert_eq!(clean_name("   \n\t "), Err(NameError::Empty));
    assert_eq!(clean_name("\u{0}\u{1}"), Err(NameError::Empty));
    let long = "é".repeat(60);
    assert_eq!(clean_name(&long).unwrap().chars().count(), MAX_NAME_CHARS);
    let spaced = format!("{}{}", "x".repeat(MAX_NAME_CHARS - 1), "    suite");
    assert!(!clean_name(&spaced).unwrap().ends_with(' '));
    assert_eq!(clean_name("🙂 Joyeux").unwrap(), "🙂 Joyeux");
}

#[test]
fn ids_are_validated() {
    assert!(valid_id("t1a2b"));
    assert!(!valid_id("t"));
    assert!(!valid_id(""));
    assert!(!valid_id("x1a2b"));
    assert!(!valid_id("t../etc"));
    assert!(!valid_id("tABC"));
    assert!(!valid_id(&format!("t{}", "a".repeat(40))));
}

#[test]
fn creating_a_theme_copies_the_source_palettes_and_keeps_its_base() {
    let (store, id, _) = store_with_one("create");
    let theme = store.get(&id).unwrap();
    assert_eq!(theme.dark, frost().dark);
    assert_eq!(theme.light, frost().light);
    assert_eq!(theme.base, BuiltinTheme::Frost);
    assert_eq!(theme.name, "Mon thème");
    assert!(valid_id(&id));
}

#[test]
fn duplicate_names_get_distinct_numbered_suffixes() {
    let dir = temp_dir("names");
    let mut store = ThemeStore::load(&dir);
    let mut names = Vec::new();
    for _ in 0..50 {
        let id = store.create(BuiltinTheme::Classic, &ResolvedTheme::classic(), "Perso");
        names.push(store.get(&id).unwrap().name.clone());
    }
    let unique: std::collections::HashSet<_> = names.iter().map(|n| n.to_lowercase()).collect();
    assert_eq!(unique.len(), 50);
    assert_eq!(names[0], "Perso");
    assert_eq!(names[1], "Perso (2)");
    assert_eq!(names[49], "Perso (50)");
}

#[test]
fn suffixes_never_break_the_length_limit() {
    let dir = temp_dir("longnames");
    let mut store = ThemeStore::load(&dir);
    let long = "a".repeat(MAX_NAME_CHARS);
    for _ in 0..12 {
        let id = store.create(BuiltinTheme::Classic, &ResolvedTheme::classic(), &long);
        assert!(store.get(&id).unwrap().name.chars().count() <= MAX_NAME_CHARS);
    }
    let names: std::collections::HashSet<_> =
        store.themes().iter().map(|t| t.name.clone()).collect();
    assert_eq!(names.len(), 12);
}

#[test]
fn names_collide_regardless_of_case() {
    let (mut store, _, _) = store_with_one("case");
    let id = store.create(BuiltinTheme::Frost, &frost(), "MON THÈME");
    assert_eq!(store.get(&id).unwrap().name, "MON THÈME (2)");
}

#[test]
fn rename_cleans_dedups_and_rejects_empty() {
    let (mut store, id, _) = store_with_one("rename");
    let other = store.create(BuiltinTheme::Frost, &frost(), "Autre");
    assert_eq!(store.rename(&id, "  Neuf \n").unwrap(), "Neuf");
    assert_eq!(store.rename(&id, "   "), Err(NameError::Empty));
    assert_eq!(store.get(&id).unwrap().name, "Neuf");
    assert_eq!(store.rename(&other, "neuf").unwrap(), "neuf (2)");
    // Garder son propre nom, à la casse près, n'ajoute pas de suffixe.
    assert_eq!(store.rename(&id, "NEUF").unwrap(), "NEUF");
}

#[test]
fn duplicate_copies_colors_and_numbers_the_name() {
    let (mut store, id, _) = store_with_one("duplicate");
    store.update(&id, |t| t.dark.accent = Color32::from_rgb(1, 2, 3));
    let copy = store.duplicate(&id).unwrap();
    assert_ne!(copy, id);
    let (a, b) = (store.get(&id).unwrap(), store.get(&copy).unwrap());
    assert_eq!(b.dark, a.dark);
    assert_eq!(b.name, "Mon thème (2)");
    assert_eq!(store.duplicate("inconnu"), None);
}

#[test]
fn every_change_bumps_the_revision() {
    let (mut store, id, _) = store_with_one("revision");
    let r0 = store.revision;
    store.update(&id, |t| t.dark.text = Color32::WHITE);
    let r1 = store.revision;
    assert!(r1 > r0);
    store.rename(&id, "Autre nom").unwrap();
    assert!(store.revision > r1);
}

#[test]
fn flush_writes_then_load_reads_the_same_themes() {
    let (mut store, id, dir) = store_with_one("flush");
    store.update(&id, |t| t.light.background = Color32::from_rgb(9, 8, 7));
    assert!(store.has_unsaved());
    store.flush().unwrap();
    assert!(!store.has_unsaved());
    assert!(dir.join(format!("{id}.json")).exists());
    assert!(
        !dir.join(format!("{id}.json.part")).exists(),
        "pas de reste d'écriture"
    );

    let reloaded = ThemeStore::load(&dir);
    assert_eq!(reloaded.themes().len(), 1);
    assert_eq!(reloaded.get(&id), store.get(&id));
}

#[test]
fn flush_creates_the_missing_directory() {
    let parent = temp_dir("mkdir");
    let dir = parent.join("themes").join("profond");
    let mut store = ThemeStore::load(&dir);
    store.create(BuiltinTheme::Classic, &ResolvedTheme::classic(), "x");
    store.flush().unwrap();
    assert_eq!(ThemeStore::load(&dir).themes().len(), 1);
}

#[test]
fn loading_a_missing_directory_gives_an_empty_store() {
    let store = ThemeStore::load(&temp_dir("absent").join("n-existe-pas"));
    assert!(store.themes().is_empty());
}

#[test]
fn unreadable_files_are_skipped_without_hiding_the_others() {
    let (mut store, id, dir) = store_with_one("corrupt");
    store.flush().unwrap();
    std::fs::write(dir.join("t999.json"), "{ pas du json").unwrap();
    std::fs::write(dir.join("t998.json"), "").unwrap();
    std::fs::write(dir.join("t997.json"), [0u8, 159, 146, 150]).unwrap();
    std::fs::write(dir.join("notes.json"), "{}").unwrap();
    std::fs::write(dir.join("t996.txt"), "x").unwrap();
    std::fs::write(
        dir.join("t995.json"),
        r#"{"abcom_theme":99,"name":"x","base":"classic"}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("t994.json"),
        vec![b' '; (MAX_FILE_BYTES + 1) as usize],
    )
    .unwrap();
    let reloaded = ThemeStore::load(&dir);
    assert_eq!(reloaded.themes().len(), 1);
    assert_eq!(reloaded.themes()[0].id, id);
}

#[test]
fn a_file_with_a_hostile_name_or_id_is_ignored() {
    let dir = temp_dir("hostile");
    let mut store = ThemeStore::load(&dir);
    let id = store.create(BuiltinTheme::Classic, &ResolvedTheme::classic(), "ok");
    store.flush().unwrap();
    let valid = std::fs::read_to_string(dir.join(format!("{id}.json"))).unwrap();
    std::fs::write(dir.join("t..x.json"), &valid).unwrap();
    std::fs::write(dir.join("T1.json"), &valid).unwrap();
    assert_eq!(ThemeStore::load(&dir).themes().len(), 1);
}

#[test]
fn a_stored_name_is_cleaned_on_load_and_an_unknown_base_becomes_classic() {
    let (mut store, id, dir) = store_with_one("clean-load");
    store.flush().unwrap();
    let path = dir.join(format!("{id}.json"));
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    json["name"] = "  Sale \n nom ".into();
    json["base"] = "disparu".into();
    std::fs::write(&path, json.to_string()).unwrap();
    let theme = ThemeStore::load(&dir).get(&id).unwrap().clone();
    assert_eq!(theme.name, "Sale nom");
    assert_eq!(theme.base, BuiltinTheme::Classic);
}

#[test]
fn a_stored_empty_name_makes_the_file_unusable() {
    let (mut store, id, dir) = store_with_one("empty-name");
    store.flush().unwrap();
    let path = dir.join(format!("{id}.json"));
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    json["name"] = "   ".into();
    std::fs::write(&path, json.to_string()).unwrap();
    assert!(ThemeStore::load(&dir).themes().is_empty());
}

#[test]
fn delete_removes_the_file_and_reports_the_base_theme() {
    let (mut store, id, dir) = store_with_one("delete");
    store.flush().unwrap();
    assert_eq!(store.delete(&id), Some(BuiltinTheme::Frost));
    assert!(store.get(&id).is_none());
    assert!(!dir.join(format!("{id}.json")).exists());
    assert_eq!(store.delete(&id), None, "déjà supprimé");
    assert!(ThemeStore::load(&dir).themes().is_empty());
}

#[test]
fn deleting_an_unsaved_theme_leaves_nothing_to_write() {
    let (mut store, id, dir) = store_with_one("delete-unsaved");
    store.delete(&id);
    assert!(!store.has_unsaved());
    store.flush().unwrap();
    assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
}

#[test]
fn deleting_a_file_removed_by_hand_does_not_fail() {
    let (mut store, id, dir) = store_with_one("delete-gone");
    store.flush().unwrap();
    std::fs::remove_file(dir.join(format!("{id}.json"))).unwrap();
    assert_eq!(store.delete(&id), Some(BuiltinTheme::Frost));
}

/// Une écriture refusée (dossier en lecture seule) ne perd pas le thème et ne
/// réessaie pas à chaque image.
#[cfg(unix)]
#[test]
fn a_read_only_directory_keeps_the_theme_in_memory_and_reports_once() {
    use std::os::unix::fs::PermissionsExt;
    let dir = temp_dir("readonly");
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).unwrap();
    // Sous root, les permissions ne protègent plus : le test n'aurait rien à prouver.
    if std::fs::write(dir.join("sonde"), "x").is_ok() {
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        return;
    }
    let mut store = ThemeStore::load(&dir);
    let id = store.create(BuiltinTheme::Frost, &frost(), "Lecture seule");
    assert!(store.flush().is_err(), "l'échec est signalé");
    assert!(store.get(&id).is_some(), "le thème reste utilisable");
    assert!(store.flush().is_ok(), "pas de nouvel échec à chaque image");
    assert!(store.has_unsaved());

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    store.update(&id, |t| t.dark.accent = Color32::from_rgb(5, 5, 5));
    store.flush().unwrap();
    assert!(!store.has_unsaved());
    assert_eq!(ThemeStore::load(&dir).themes().len(), 1);
}

#[test]
fn a_custom_theme_resolves_with_its_palettes_base_typography_and_styled_widgets() {
    let (mut store, id, _) = store_with_one("resolve");
    store.update(&id, |t| t.dark.accent = Color32::from_rgb(10, 20, 30));
    let resolved = store.get(&id).unwrap().resolve(&TextSettings::default());
    assert_eq!(resolved.dark.accent, Color32::from_rgb(10, 20, 30));
    assert!(resolved.styled_widgets);
    assert_eq!(resolved.text.body, 13.0, "typographie de Givre");
    let lounge = CustomTheme {
        base: BuiltinTheme::Lounge,
        ..store.get(&id).unwrap().clone()
    };
    assert_eq!(lounge.resolve(&TextSettings::default()).text.body, 15.0);
}

#[test]
fn palette_mut_picks_the_requested_mode() {
    let (store, id, _) = store_with_one("palette-mut");
    let mut theme = store.get(&id).unwrap().clone();
    theme.palette_mut(true).text = Color32::from_rgb(1, 1, 1);
    theme.palette_mut(false).text = Color32::from_rgb(2, 2, 2);
    assert_eq!(theme.dark.text, Color32::from_rgb(1, 1, 1));
    assert_eq!(theme.light.text, Color32::from_rgb(2, 2, 2));
}

#[test]
fn themes_are_listed_by_name_after_a_reload() {
    let dir = temp_dir("sorted");
    let mut store = ThemeStore::load(&dir);
    for name in ["Zèbre", "alpha", "Mimosa"] {
        store.create(BuiltinTheme::Classic, &ResolvedTheme::classic(), name);
    }
    store.flush().unwrap();
    let names: Vec<_> = ThemeStore::load(&dir)
        .themes()
        .iter()
        .map(|t| t.name.clone())
        .collect();
    assert_eq!(names, ["alpha", "Mimosa", "Zèbre"]);
}
