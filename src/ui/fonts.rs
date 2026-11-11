//! Polices choisies par l'utilisateur : proposées, importées depuis un fichier.
//!
//! egui panique sur une police qu'il ne sait pas lire : tout ce qui n'est pas
//! embarqué en dur est donc décodé puis validé ici, avant d'être confié à egui.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};
use skrifa::MetadataProvider;

use super::theme::FontKind;

/// Au-delà, un fichier de police n'en est probablement pas une.
pub(crate) const MAX_FONT_FILE_BYTES: u64 = 30 * 1024 * 1024;
/// Taille maximale d'une police une fois décompressée (protège des bombes de décompression).
pub(crate) const MAX_DECODED_BYTES: u32 = 60 * 1024 * 1024;
/// Nombre de glyphes sondés pour s'assurer qu'une police dessine en vectoriel.
const OUTLINE_PROBE: usize = 512;
/// Lettres qu'une police de texte latin doit dessiner : une police de symboles en porte parfois une ou deux.
const LATIN_PROBE: &str = "aeinorstu";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontError {
    /// Le fichier n'est pas une police lisible.
    Unreadable,
    /// Aucune lettre latine : une police de symboles ou d'une autre écriture seule.
    NoLatin,
    /// Pas de contours vectoriels : police bitmap ou en couleur.
    NoOutline,
    /// Fichier ou police décompressée trop lourds.
    TooLarge,
}

/// Décompresse un WOFF ou un WOFF2 ; un TTF, un OTF ou un TTC passent tels quels.
pub(crate) fn decode_font(raw: &[u8]) -> Result<Vec<u8>, FontError> {
    match raw.get(..4) {
        Some(b"wOF2") | Some(b"wOFF") => {
            // `totalSfntSize` : même place (octet 16) dans les deux formats.
            let announced = raw
                .get(16..20)
                .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
                .ok_or(FontError::Unreadable)?;
            if announced > MAX_DECODED_BYTES {
                return Err(FontError::TooLarge);
            }
            let decoded = if raw.starts_with(b"wOF2") {
                wuff::decompress_woff2(raw)
            } else {
                wuff::decompress_woff1(raw)
            };
            let decoded = decoded.map_err(|_| FontError::Unreadable)?;
            if decoded.len() as u64 > u64::from(MAX_DECODED_BYTES) {
                return Err(FontError::TooLarge);
            }
            Ok(decoded)
        }
        _ => Ok(raw.to_vec()),
    }
}

/// Vérifie qu'un ensemble de fichiers forme une police utilisable et en rend le nom de famille.
///
/// Chaque fichier doit être lisible et dessiner au moins un glyphe vectoriel.
/// L'ensemble doit dessiner les lettres de base : un sous-ensemble `latin-ext` n'en contient
/// pas, et reste valable à côté de son `latin`.
pub(crate) fn validate_font_set(parts: &[(&[u8], u32)]) -> Result<String, FontError> {
    if parts.is_empty() {
        return Err(FontError::Unreadable);
    }
    let mut family = None;
    let mut has_latin = false;
    for (bytes, index) in parts {
        let font = skrifa::FontRef::from_index(bytes, *index).map_err(|_| FontError::Unreadable)?;
        let outlines = font.outline_glyphs();
        let charmap = font.charmap();
        let draws = charmap
            .mappings()
            .take(OUTLINE_PROBE)
            .any(|(_, glyph)| outlines.get(glyph).is_some());
        if !draws {
            return Err(FontError::NoOutline);
        }
        has_latin |= LATIN_PROBE.chars().all(|c| {
            charmap
                .map(c)
                .and_then(|glyph| outlines.get(glyph))
                .is_some()
        });
        family = family.or_else(|| {
            font.localized_strings(skrifa::string::StringId::FAMILY_NAME)
                .english_or_first()
                .map(|name| name.chars().collect::<String>())
        });
    }
    if !has_latin {
        return Err(FontError::NoLatin);
    }
    Ok(family
        .filter(|f| !f.trim().is_empty())
        .unwrap_or_else(|| "Police".to_owned()))
}

/// Police proposée dans l'application, embarquée au format WOFF2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SuggestedFont {
    Atkinson,
    Lexend,
    OpenDyslexic,
    Literata,
    Nunito,
    JetBrainsMono,
}

macro_rules! suggestion_files {
    ($dir:literal, [$($file:literal),*]) => {
        &[$(include_bytes!(concat!("../../assets/fonts/suggestions/", $dir, "/", $file, ".woff2")) as &[u8]),*]
    };
}

impl SuggestedFont {
    pub(crate) const ALL: [Self; 6] = [
        Self::Atkinson,
        Self::Lexend,
        Self::OpenDyslexic,
        Self::Literata,
        Self::Nunito,
        Self::JetBrainsMono,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Atkinson => "atkinson-hyperlegible",
            Self::Lexend => "lexend",
            Self::OpenDyslexic => "opendyslexic",
            Self::Literata => "literata",
            Self::Nunito => "nunito",
            Self::JetBrainsMono => "jetbrains-mono",
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|font| font.id() == id)
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Atkinson => "Atkinson Hyperlegible",
            Self::Lexend => "Lexend",
            Self::OpenDyslexic => "OpenDyslexic",
            Self::Literata => "Literata",
            Self::Nunito => "Nunito",
            Self::JetBrainsMono => "JetBrains Mono",
        }
    }

    /// Fichiers WOFF2 de la coupe normale.
    fn regular_files(self) -> &'static [&'static [u8]] {
        match self {
            Self::Atkinson => {
                suggestion_files!("atkinson-hyperlegible", ["latin-400", "latin-ext-400"])
            }
            Self::Lexend => suggestion_files!("lexend", ["latin-400", "latin-ext-400"]),
            Self::OpenDyslexic => suggestion_files!("opendyslexic", ["latin-400"]),
            Self::Literata => suggestion_files!("literata", ["latin-400", "latin-ext-400"]),
            Self::Nunito => suggestion_files!("nunito", ["latin-400", "latin-ext-400"]),
            Self::JetBrainsMono => {
                suggestion_files!("jetbrains-mono", ["latin-400", "latin-ext-400"])
            }
        }
    }

    /// Fichiers WOFF2 de la coupe grasse.
    fn bold_files(self) -> &'static [&'static [u8]] {
        match self {
            Self::Atkinson => {
                suggestion_files!("atkinson-hyperlegible", ["latin-700", "latin-ext-700"])
            }
            Self::Lexend => suggestion_files!("lexend", ["latin-700", "latin-ext-700"]),
            Self::OpenDyslexic => suggestion_files!("opendyslexic", ["latin-700"]),
            Self::Literata => suggestion_files!("literata", ["latin-700", "latin-ext-700"]),
            Self::Nunito => suggestion_files!("nunito", ["latin-700", "latin-ext-700"]),
            Self::JetBrainsMono => {
                suggestion_files!("jetbrains-mono", ["latin-700", "latin-ext-700"])
            }
        }
    }

    /// Polices décodées, gardées en mémoire une fois pour toutes.
    pub(crate) fn decoded(self) -> Result<DecodedFont, FontError> {
        static CACHE: OnceLock<Mutex<std::collections::HashMap<SuggestedFont, DecodedFont>>> =
            OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some(found) = cache.lock().ok().and_then(|c| c.get(&self).cloned()) {
            return Ok(found);
        }
        let decode_all = |files: &[&[u8]]| -> Result<Vec<Arc<Vec<u8>>>, FontError> {
            files
                .iter()
                .map(|raw| decode_font(raw).map(Arc::new))
                .collect()
        };
        let font = DecodedFont {
            regular: decode_all(self.regular_files())?,
            bold: decode_all(self.bold_files())?,
        };
        if let Ok(mut cache) = cache.lock() {
            cache.insert(self, font.clone());
        }
        Ok(font)
    }
}

/// Fichiers d'une police prêts pour egui : un ou plusieurs sous-ensembles par coupe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedFont {
    pub(crate) regular: Vec<Arc<Vec<u8>>>,
    pub(crate) bold: Vec<Arc<Vec<u8>>>,
}

/// Choix de police d'une zone (interface ou conversations).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum FontChoice {
    /// Celle du thème.
    #[default]
    ThemeDefault,
    NotoSans,
    Inter,
    Suggested(SuggestedFont),
    Imported(String),
}

impl FontChoice {
    pub(crate) fn to_id(&self) -> String {
        match self {
            Self::ThemeDefault => "theme".to_owned(),
            Self::NotoSans => "noto-sans".to_owned(),
            Self::Inter => "inter".to_owned(),
            Self::Suggested(font) => format!("suggested:{}", font.id()),
            Self::Imported(id) => format!("imported:{id}"),
        }
    }

    /// Relit un identifiant ; un identifiant inconnu donne la police du thème.
    pub(crate) fn from_id(id: &str) -> Self {
        match id {
            "noto-sans" => Self::NotoSans,
            "inter" => Self::Inter,
            _ => {
                if let Some(font) = id
                    .strip_prefix("suggested:")
                    .and_then(SuggestedFont::from_id)
                {
                    Self::Suggested(font)
                } else if let Some(imported) =
                    id.strip_prefix("imported:").filter(|i| valid_font_id(i))
                {
                    Self::Imported(imported.to_owned())
                } else {
                    Self::ThemeDefault
                }
            }
        }
    }
}

impl Serialize for FontChoice {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_id())
    }
}

impl<'de> Deserialize<'de> for FontChoice {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_id(&String::deserialize(deserializer)?))
    }
}

fn valid_font_id(id: &str) -> bool {
    id.len() > 1
        && id.len() <= 20
        && id.starts_with('f')
        && id[1..]
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Ce que l'interface et les conversations utilisent pour dessiner le texte.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FontPlan {
    /// Une police embarquée en dur.
    Theme(FontKind),
    /// Une police décodée : proposée, importée ou téléchargée.
    Files { key: String, font: DecodedFont },
}

impl FontPlan {
    /// Identifie la police : change quand les octets changent.
    pub(crate) fn key(&self) -> String {
        match self {
            Self::Theme(kind) => format!("theme:{kind:?}"),
            Self::Files { key, .. } => key.clone(),
        }
    }

    pub(crate) fn real_bold(&self) -> bool {
        match self {
            Self::Theme(kind) => *kind == FontKind::Inter,
            Self::Files { font, .. } => !font.bold.is_empty(),
        }
    }

    pub(crate) fn real_italic(&self) -> bool {
        matches!(self, Self::Theme(FontKind::Inter))
    }
}

/// Polices des deux zones.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FontPlans {
    pub(crate) ui: FontPlan,
    pub(crate) msg: FontPlan,
}

impl FontPlans {
    pub(crate) fn theme(kind: FontKind) -> Self {
        Self {
            ui: FontPlan::Theme(kind),
            msg: FontPlan::Theme(kind),
        }
    }

    pub(crate) fn key(&self) -> String {
        format!("{}|{}", self.ui.key(), self.msg.key())
    }
}

/// Police d'une zone à partir du choix de l'utilisateur.
pub(crate) fn resolve_choice(
    choice: &FontChoice,
    theme_font: FontKind,
    store: &FontStore,
) -> Result<FontPlan, FontError> {
    Ok(match choice {
        FontChoice::ThemeDefault => FontPlan::Theme(theme_font),
        FontChoice::NotoSans => FontPlan::Theme(FontKind::NotoSans),
        FontChoice::Inter => FontPlan::Theme(FontKind::Inter),
        FontChoice::Suggested(font) => FontPlan::Files {
            key: format!("suggested:{}", font.id()),
            font: font.decoded()?,
        },
        FontChoice::Imported(id) => {
            let hash = store
                .get(id)
                .map(|f| f.hash.clone())
                .ok_or(FontError::Unreadable)?;
            FontPlan::Files {
                key: format!("imported:{id}:{hash}"),
                font: store.read_font(id)?,
            }
        }
    })
}

/// Police importée par l'utilisateur.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ImportedFont {
    pub(crate) id: String,
    /// Nom de famille lu dans le fichier, éventuellement suivi de « (2) ».
    pub(crate) family: String,
    /// Empreinte du contenu décodé, pour ne pas importer deux fois le même fichier.
    pub(crate) hash: String,
    /// Fichier TTF ou TTC dans le dossier des polices.
    pub(crate) file: String,
    /// Autres fichiers de la coupe normale (sous-ensembles latin-ext…).
    #[serde(default)]
    pub(crate) extra: Vec<String>,
    /// Fichiers de la coupe grasse, quand la police en a une.
    #[serde(default)]
    pub(crate) bold: Vec<String>,
    /// D'où vient la police : `fichier` ou `fontsource:<id>`.
    #[serde(default)]
    pub(crate) source: String,
}

impl ImportedFont {
    fn all_files(&self) -> impl Iterator<Item = &String> {
        std::iter::once(&self.file)
            .chain(&self.extra)
            .chain(&self.bold)
    }
}

/// Dossier des polices importées et leur index.
#[derive(Debug)]
pub(crate) struct FontStore {
    dir: PathBuf,
    fonts: Vec<ImportedFont>,
    /// Change à chaque ajout ou retrait.
    pub(crate) revision: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ImportOutcome {
    Added(ImportedFont),
    /// Le même contenu était déjà importé.
    Existing(ImportedFont),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ImportError {
    Font(FontError),
    /// Lecture ou écriture impossible.
    Io(String),
}

impl FontStore {
    const INDEX: &'static str = "index.json";

    /// Lit l'index ; les entrées dont le fichier manque ou est illisible sont écartées.
    pub(crate) fn load(dir: &Path) -> Self {
        let raw = std::fs::read_to_string(dir.join(Self::INDEX)).unwrap_or_default();
        let mut fonts: Vec<ImportedFont> = serde_json::from_str(&raw).unwrap_or_default();
        fonts.retain(|font| {
            let ok = valid_font_id(&font.id)
                && font.file == format!("{}.ttf", font.id)
                && font.all_files().all(|file| {
                    file.starts_with(&font.id)
                        && file.ends_with(".ttf")
                        && !file.contains(['/', '\\'])
                        && dir.join(file).is_file()
                });
            if !ok {
                tracing::warn!("police importée {} ignorée", font.id);
            }
            ok
        });
        Self {
            dir: dir.to_owned(),
            fonts,
            revision: 0,
        }
    }

    /// Dossier des polices.
    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(crate) fn fonts(&self) -> &[ImportedFont] {
        &self.fonts
    }

    pub(crate) fn get(&self, id: &str) -> Option<&ImportedFont> {
        self.fonts.iter().find(|f| f.id == id)
    }

    /// Octets du fichier principal d'une police importée, relus et revalidés.
    #[cfg(test)]
    pub(crate) fn read(&self, id: &str) -> Result<Arc<Vec<u8>>, FontError> {
        self.read_font(id).map(|font| font.regular[0].clone())
    }

    /// Tous les fichiers d'une police importée, relus et revalidés : un fichier
    /// remplacé à la main ne doit pas atteindre egui.
    pub(crate) fn read_font(&self, id: &str) -> Result<DecodedFont, FontError> {
        let font = self.get(id).ok_or(FontError::Unreadable)?;
        let load =
            |files: &mut dyn Iterator<Item = &String>| -> Result<Vec<Arc<Vec<u8>>>, FontError> {
                files
                    .map(|file| {
                        std::fs::read(self.dir.join(file))
                            .map(Arc::new)
                            .map_err(|_| FontError::Unreadable)
                    })
                    .collect()
            };
        let regular = load(&mut std::iter::once(&font.file).chain(&font.extra))?;
        let bold = load(&mut font.bold.iter())?;
        let check = |files: &[Arc<Vec<u8>>]| {
            let parts: Vec<(&[u8], u32)> = files.iter().map(|f| (f.as_slice(), 0)).collect();
            validate_font_set(&parts)
        };
        check(&regular)?;
        if !bold.is_empty() {
            check(&bold)?;
        }
        Ok(DecodedFont { regular, bold })
    }

    /// Importe un fichier : taille, décodage, validation, puis copie.
    pub(crate) fn import(&mut self, source: &Path) -> Result<ImportOutcome, ImportError> {
        let meta = std::fs::metadata(source).map_err(|e| ImportError::Io(e.to_string()))?;
        if meta.len() > MAX_FONT_FILE_BYTES {
            return Err(ImportError::Font(FontError::TooLarge));
        }
        let raw = std::fs::read(source).map_err(|e| ImportError::Io(e.to_string()))?;
        self.import_bytes(&raw)
    }

    pub(crate) fn import_bytes(&mut self, raw: &[u8]) -> Result<ImportOutcome, ImportError> {
        if raw.len() as u64 > MAX_FONT_FILE_BYTES {
            return Err(ImportError::Font(FontError::TooLarge));
        }
        self.import_set(&[raw], &[], "fichier")
    }

    /// Importe une police en un ou plusieurs fichiers : la coupe normale (au
    /// moins un fichier) et, si elle existe, la grasse. `source` identifie
    /// l'origine, pour ne pas la télécharger deux fois.
    pub(crate) fn import_set(
        &mut self,
        regular: &[&[u8]],
        bold: &[&[u8]],
        source: &str,
    ) -> Result<ImportOutcome, ImportError> {
        if source != "fichier" {
            if let Some(existing) = self.fonts.iter().find(|f| f.source == source) {
                return Ok(ImportOutcome::Existing(existing.clone()));
            }
        }
        let decode = |files: &[&[u8]]| -> Result<Vec<Vec<u8>>, ImportError> {
            files
                .iter()
                .map(|raw| decode_font(raw).map_err(ImportError::Font))
                .collect()
        };
        let regular = decode(regular)?;
        let bold = decode(bold)?;
        let parts = |files: &[Vec<u8>]| -> Vec<(Vec<u8>, u32)> {
            files.iter().map(|f| (f.clone(), 0)).collect()
        };
        let check = |files: &[Vec<u8>]| {
            let owned = parts(files);
            let borrowed: Vec<(&[u8], u32)> =
                owned.iter().map(|(b, i)| (b.as_slice(), *i)).collect();
            validate_font_set(&borrowed).map_err(ImportError::Font)
        };
        let family = check(&regular)?;
        if !bold.is_empty() {
            check(&bold)?;
        }
        let hash = hex_digest(&regular.concat());
        if let Some(existing) = self.fonts.iter().find(|f| f.hash == hash) {
            return Ok(ImportOutcome::Existing(existing.clone()));
        }
        let id = self.new_id();
        let names = |prefix: &str, count: usize| -> Vec<String> {
            (0..count).map(|n| format!("{id}{prefix}{n}.ttf")).collect()
        };
        let file = format!("{id}.ttf");
        let extra = names("-", regular.len().saturating_sub(1))
            .into_iter()
            .enumerate()
            .map(|(n, _)| format!("{id}-{}.ttf", n + 1))
            .collect::<Vec<_>>();
        let bold_files = names("-b", bold.len());
        std::fs::create_dir_all(&self.dir).map_err(|e| ImportError::Io(e.to_string()))?;
        let mut written: Vec<String> = Vec::new();
        let outputs = std::iter::once(&file)
            .chain(&extra)
            .zip(&regular)
            .chain(bold_files.iter().zip(&bold));
        for (name, bytes) in outputs {
            if let Err(err) = write_atomic(&self.dir.join(name), bytes) {
                for done in &written {
                    let _ = std::fs::remove_file(self.dir.join(done));
                }
                return Err(ImportError::Io(err));
            }
            written.push(name.clone());
        }
        let font = ImportedFont {
            id,
            family: self.unique_family(&family),
            hash,
            file,
            extra,
            bold: bold_files,
            source: source.to_owned(),
        };
        self.fonts.push(font.clone());
        if let Err(err) = self.save_index() {
            // Sans index, les fichiers seraient orphelins au prochain lancement.
            self.fonts.pop();
            for done in &written {
                let _ = std::fs::remove_file(self.dir.join(done));
            }
            return Err(ImportError::Io(err));
        }
        self.revision += 1;
        Ok(ImportOutcome::Added(font))
    }

    /// Retire une police et son fichier.
    pub(crate) fn remove(&mut self, id: &str) -> bool {
        let Some(index) = self.fonts.iter().position(|f| f.id == id) else {
            return false;
        };
        let font = self.fonts.remove(index);
        for file in font.all_files() {
            let _ = std::fs::remove_file(self.dir.join(file));
        }
        if let Err(err) = self.save_index() {
            tracing::warn!("index des polices non réécrit : {err}");
        }
        self.revision += 1;
        true
    }

    fn unique_family(&self, family: &str) -> String {
        let taken = |name: &str| self.fonts.iter().any(|f| f.family == name);
        if !taken(family) {
            return family.to_owned();
        }
        (2..)
            .map(|n| format!("{family} ({n})"))
            .find(|name| !taken(name))
            .unwrap_or_else(|| family.to_owned())
    }

    fn new_id(&self) -> String {
        let mut counter = 0u64;
        loop {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let id = format!("f{:x}{:x}", millis & 0xffff_ffff_ffff, counter);
            if self.get(&id).is_none() && valid_font_id(&id) {
                return id;
            }
            counter += 1;
        }
    }

    fn save_index(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.fonts).map_err(|e| e.to_string())?;
        write_atomic(&self.dir.join(Self::INDEX), json.as_bytes())
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    Blake2s256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Écrit dans un fichier voisin puis le renomme : jamais de fichier tronqué.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let part = path.with_extension("part");
    std::fs::write(&part, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&part, path).map_err(|e| {
        let _ = std::fs::remove_file(&part);
        e.to_string()
    })
}

#[cfg(test)]
#[path = "../tests/test_ui_fonts_store.rs"]
mod tests;
