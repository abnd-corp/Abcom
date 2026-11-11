//! Thèmes personnels : copies d'un thème intégré dont on a changé les couleurs.
//!
//! Chaque thème est un fichier `<id>.json` du dossier `themes/` des données.
//! Un fichier illisible est ignoré sans toucher aux autres, et une écriture qui
//! échoue (disque plein, dossier en lecture seule) laisse le thème utilisable
//! pour la session.

// Création, copie et édition sont appelées par l'onglet Apparence (commit suivant).
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

use super::appearance::TextSettings;
use super::theme::{Palette, ResolvedTheme, TextMetrics};
use super::themes::BuiltinTheme;

/// Préfixe d'un thème personnel dans [`super::appearance::Appearance::theme`].
pub(crate) const CUSTOM_PREFIX: &str = "custom:";
/// Version du format de fichier.
const FORMAT_VERSION: u32 = 1;
/// Longueur maximale du nom d'un thème, en caractères.
pub(crate) const MAX_NAME_CHARS: usize = 40;
/// Au-delà, un fichier de thème n'en est pas un.
const MAX_FILE_BYTES: u64 = 256 * 1024;

macro_rules! palette_data {
    ($($field:ident),* $(,)?) => {
        /// Palette sous forme enregistrable : quatre octets sRGB par rôle.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        pub(crate) struct PaletteData {
            $(pub(crate) $field: [u8; 4],)*
        }

        impl From<Palette> for PaletteData {
            fn from(palette: Palette) -> Self {
                Self { $($field: palette.$field.to_srgba_unmultiplied(),)* }
            }
        }

        impl From<PaletteData> for Palette {
            fn from(data: PaletteData) -> Self {
                Self {
                    $($field: Color32::from_rgba_unmultiplied(
                        data.$field[0], data.$field[1], data.$field[2], data.$field[3],
                    ),)*
                }
            }
        }
    };
}

palette_data!(
    surface,
    surface_strong,
    surface_hover,
    separator,
    text,
    text_muted,
    accent,
    accent_soft,
    danger,
    success,
    receipt_read,
    link,
    block_bg,
    block_stripe,
    block_border,
    block_text,
    background,
    input_bg,
    chip,
    chip_mine,
    chip_mine_border,
    row_tint,
);

#[derive(Serialize, Deserialize)]
struct ThemeFile {
    abcom_theme: u32,
    name: String,
    base: String,
    dark: PaletteData,
    light: PaletteData,
}

/// Thème personnel chargé en mémoire.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CustomTheme {
    pub(crate) id: String,
    pub(crate) name: String,
    /// Thème intégré dont il est tiré : il en garde la typographie.
    pub(crate) base: BuiltinTheme,
    pub(crate) dark: Palette,
    pub(crate) light: Palette,
}

impl CustomTheme {
    pub(crate) fn resolve(&self, settings: &TextSettings) -> ResolvedTheme {
        ResolvedTheme {
            revision: 0,
            dark: self.dark,
            light: self.light,
            text: TextMetrics::resolve(&self.base.typography(), settings),
            styled_widgets: true,
        }
    }

    /// Palette modifiable du mode demandé.
    pub(crate) fn palette_mut(&mut self, dark_mode: bool) -> &mut Palette {
        if dark_mode {
            &mut self.dark
        } else {
            &mut self.light
        }
    }
}

/// Pourquoi un nom est refusé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NameError {
    Empty,
}

/// Nettoie un nom : retours à la ligne et caractères de contrôle deviennent des
/// espaces, les blancs de bord et les doublons disparaissent, la longueur est bornée.
pub(crate) fn clean_name(raw: &str) -> Result<String, NameError> {
    let spaced: String = raw
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let joined = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let clipped: String = joined.chars().take(MAX_NAME_CHARS).collect();
    let clipped = clipped.trim_end().to_owned();
    if clipped.is_empty() {
        Err(NameError::Empty)
    } else {
        Ok(clipped)
    }
}

fn valid_id(id: &str) -> bool {
    id.len() > 1
        && id.len() <= 20
        && id.starts_with('t')
        && id[1..]
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Dossier des thèmes personnels, avec leur état en mémoire.
#[derive(Debug)]
pub(crate) struct ThemeStore {
    dir: PathBuf,
    themes: Vec<CustomTheme>,
    /// Change à chaque modification, pour que l'apparence soit réappliquée.
    pub(crate) revision: u64,
    dirty: BTreeSet<String>,
    /// Une écriture a échoué : on ne réessaie qu'après un nouveau changement.
    write_failed: bool,
    counter: u64,
}

impl ThemeStore {
    /// Lit les thèmes du dossier ; ceux qu'on ne peut pas lire sont ignorés.
    pub(crate) fn load(dir: &Path) -> Self {
        let mut themes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                match read_theme(&path) {
                    Ok(theme) => themes.push(theme),
                    Err(reason) => {
                        tracing::warn!("thème {} ignoré : {reason}", path.display());
                    }
                }
            }
        }
        themes.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Self {
            dir: dir.to_owned(),
            themes,
            revision: 0,
            dirty: BTreeSet::new(),
            write_failed: false,
            counter: 0,
        }
    }

    pub(crate) fn themes(&self) -> &[CustomTheme] {
        &self.themes
    }

    pub(crate) fn get(&self, id: &str) -> Option<&CustomTheme> {
        self.themes.iter().find(|t| t.id == id)
    }

    fn changed(&mut self) {
        self.revision += 1;
        self.write_failed = false;
    }

    fn new_id(&mut self) -> String {
        loop {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            self.counter += 1;
            let id = format!("t{:x}{:x}", millis & 0xffff_ffff_ffff, self.counter);
            if self.get(&id).is_none() && valid_id(&id) {
                return id;
            }
        }
    }

    /// Nom libre : « Nom », puis « Nom (2) », « Nom (3) »… sans casser la limite de longueur.
    pub(crate) fn unique_name(&self, wanted: &str) -> String {
        let base = clean_name(wanted).unwrap_or_else(|_| "Thème".to_owned());
        let taken = |name: &str| {
            self.themes
                .iter()
                .any(|t| t.name.to_lowercase() == name.to_lowercase())
        };
        if !taken(&base) {
            return base;
        }
        for n in 2.. {
            let suffix = format!(" ({n})");
            let room = MAX_NAME_CHARS.saturating_sub(suffix.chars().count());
            let stem: String = base.chars().take(room).collect();
            let candidate = format!("{}{suffix}", stem.trim_end());
            if !taken(&candidate) {
                return candidate;
            }
        }
        unreachable!("la boucle ne se termine que par un nom libre")
    }

    /// Crée un thème à partir d'un thème résolu et renvoie son identifiant.
    pub(crate) fn create(
        &mut self,
        base: BuiltinTheme,
        from: &ResolvedTheme,
        name: &str,
    ) -> String {
        let id = self.new_id();
        let name = self.unique_name(name);
        self.themes.push(CustomTheme {
            id: id.clone(),
            name,
            base,
            dark: from.dark,
            light: from.light,
        });
        self.dirty.insert(id.clone());
        self.changed();
        id
    }

    /// Copie d'un thème personnel ; `None` si l'identifiant est inconnu.
    pub(crate) fn duplicate(&mut self, id: &str) -> Option<String> {
        let source = self.get(id)?.clone();
        let name = self.unique_name(&source.name);
        let new_id = self.new_id();
        self.themes.push(CustomTheme {
            id: new_id.clone(),
            name,
            ..source
        });
        self.dirty.insert(new_id.clone());
        self.changed();
        Some(new_id)
    }

    /// Renomme un thème ; un nom déjà pris reçoit un suffixe. Renvoie le nom retenu.
    pub(crate) fn rename(&mut self, id: &str, raw: &str) -> Result<String, NameError> {
        let cleaned = clean_name(raw)?;
        let index = self.themes.iter().position(|t| t.id == id);
        let Some(index) = index else {
            return Ok(cleaned);
        };
        let current = self.themes[index].name.clone();
        let name = if current.to_lowercase() == cleaned.to_lowercase() {
            cleaned
        } else {
            self.unique_name(&cleaned)
        };
        self.themes[index].name = name.clone();
        self.dirty.insert(id.to_owned());
        self.changed();
        Ok(name)
    }

    /// Modifie un thème ; il sera écrit au prochain [`Self::flush`].
    pub(crate) fn update(&mut self, id: &str, edit: impl FnOnce(&mut CustomTheme)) {
        if let Some(theme) = self.themes.iter_mut().find(|t| t.id == id) {
            edit(theme);
            self.dirty.insert(id.to_owned());
            self.changed();
        }
    }

    /// Supprime un thème, fichier compris. Renvoie le thème intégré dont il était tiré.
    pub(crate) fn delete(&mut self, id: &str) -> Option<BuiltinTheme> {
        let index = self.themes.iter().position(|t| t.id == id)?;
        let removed = self.themes.remove(index);
        self.dirty.remove(id);
        if let Err(err) = std::fs::remove_file(self.path(id)) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("suppression du thème {id} : {err}");
            }
        }
        self.changed();
        Some(removed.base)
    }

    fn path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    /// Écrit les thèmes modifiés. `Err` si l'écriture échoue : les thèmes restent
    /// valables en mémoire, et aucune nouvelle tentative n'a lieu avant le
    /// prochain changement.
    pub(crate) fn flush(&mut self) -> Result<(), String> {
        if self.dirty.is_empty() || self.write_failed {
            return Ok(());
        }
        std::fs::create_dir_all(&self.dir).map_err(|e| {
            self.write_failed = true;
            e.to_string()
        })?;
        for id in std::mem::take(&mut self.dirty) {
            let Some(theme) = self.get(&id) else { continue };
            if let Err(err) = write_theme(&self.path(&id), theme) {
                self.dirty.insert(id);
                self.write_failed = true;
                return Err(err);
            }
        }
        Ok(())
    }

    /// Des thèmes attendent-ils d'être écrits ?
    pub(crate) fn has_unsaved(&self) -> bool {
        !self.dirty.is_empty()
    }
}

fn read_theme(path: &Path) -> Result<CustomTheme, String> {
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| valid_id(s))
        .ok_or("nom de fichier inattendu")?
        .to_owned();
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > MAX_FILE_BYTES {
        return Err("fichier trop gros".into());
    }
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: ThemeFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if file.abcom_theme != FORMAT_VERSION {
        return Err(format!("version {} non prise en charge", file.abcom_theme));
    }
    Ok(CustomTheme {
        id,
        name: clean_name(&file.name).map_err(|_| "nom vide".to_owned())?,
        base: BuiltinTheme::from_id(&file.base).unwrap_or(BuiltinTheme::Classic),
        dark: file.dark.into(),
        light: file.light.into(),
    })
}

/// Écrit d'abord dans un fichier voisin, puis le renomme : un arrêt en cours
/// d'écriture ne laisse jamais un thème tronqué.
fn write_theme(path: &Path, theme: &CustomTheme) -> Result<(), String> {
    let file = ThemeFile {
        abcom_theme: FORMAT_VERSION,
        name: theme.name.clone(),
        base: theme.base.id().to_owned(),
        dark: theme.dark.into(),
        light: theme.light.into(),
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    let part = path.with_extension("json.part");
    std::fs::write(&part, json).map_err(|e| e.to_string())?;
    std::fs::rename(&part, path).map_err(|e| {
        let _ = std::fs::remove_file(&part);
        e.to_string()
    })
}

#[cfg(test)]
#[path = "../tests/test_ui_custom_themes.rs"]
mod tests;
