//! Catalogue de polices Fontsource : liste (API) et fichiers WOFF2 (CDN jsDelivr).
//!
//! Rien ici ne part tout seul : l'appelant doit avoir reçu l'accord explicite de
//! l'utilisateur. Les deux serveurs voient son adresse IP, rien d'autre n'est envoyé.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub const LIST_URL: &str = "https://api.fontsource.org/v1/fonts";
const CDN: &str = "https://cdn.jsdelivr.net/fontsource/fonts";
/// Durée de validité de la liste enregistrée.
pub const CACHE_TTL_SECS: u64 = 7 * 24 * 3600;
/// Taille maximale acceptée pour la liste (la vraie fait environ 1 Mo).
pub const MAX_LIST_BYTES: usize = 8 * 1024 * 1024;
/// Taille maximale d'un fichier de police téléchargé.
pub const MAX_FILE_BYTES: usize = 6 * 1024 * 1024;
/// Entrées par page.
pub const PAGE_SIZE: usize = 50;
/// Licences acceptées : celles qui autorisent de redistribuer et d'embarquer la police.
const LICENSES: [&str; 2] = ["OFL-1.1", "Apache-2.0"];

/// Une police du catalogue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogFont {
    pub id: String,
    pub family: String,
    #[serde(default)]
    pub subsets: Vec<String>,
    #[serde(default)]
    pub weights: Vec<u16>,
    #[serde(default)]
    pub styles: Vec<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub license: String,
}

/// Rôle d'un fichier dans une police téléchargée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Latin,
    LatinExt,
    BoldLatin,
    BoldLatinExt,
}

impl Part {
    pub fn is_bold(self) -> bool {
        matches!(self, Self::BoldLatin | Self::BoldLatinExt)
    }

    /// Sans lui, la police n'a pas de lettres : le téléchargement échoue.
    pub fn is_required(self) -> bool {
        self == Self::Latin
    }
}

/// Identifiant sûr : lettres, chiffres et tirets, comme dans le catalogue.
pub fn valid_font_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Liste filtrée : licences libres seulement, identifiants sûrs, doublons écartés.
pub fn parse_catalog(raw: &[u8]) -> Result<Vec<CatalogFont>, String> {
    if raw.len() > MAX_LIST_BYTES {
        return Err("liste trop grosse".into());
    }
    let values: Vec<serde_json::Value> =
        serde_json::from_slice(raw).map_err(|e| format!("liste illisible : {e}"))?;
    let mut seen = std::collections::HashSet::new();
    let mut fonts: Vec<CatalogFont> = values
        .into_iter()
        .filter_map(|value| serde_json::from_value::<CatalogFont>(value).ok())
        .filter(|font| {
            LICENSES.contains(&font.license.as_str())
                && valid_font_id(&font.id)
                && !font.family.trim().is_empty()
                && font.subsets.iter().any(|s| s == "latin")
                && seen.insert(font.id.clone())
        })
        .collect();
    fonts.sort_by_key(|font| fold(&font.family));
    if fonts.is_empty() {
        return Err("aucune police exploitable".into());
    }
    Ok(fonts)
}

/// Minuscules sans accents, pour une recherche qui n'y est sensible ni l'un ni l'autre.
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
            'ç' | 'ć' | 'č' => 'c',
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
            'ñ' | 'ń' | 'ň' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ő' => 'o',
            'ś' | 'š' | 'ş' => 's',
            'ù' | 'ú' | 'û' | 'ü' | 'ů' | 'ű' => 'u',
            'ý' | 'ÿ' => 'y',
            'ź' | 'ż' | 'ž' => 'z',
            other => other,
        })
        .collect()
}

/// Polices dont le nom contient `query` (sans tenir compte de la casse ni des accents)
/// et qui sont de la catégorie demandée (`None` = toutes).
pub fn filter<'a>(
    fonts: &'a [CatalogFont],
    query: &str,
    category: Option<&str>,
) -> Vec<&'a CatalogFont> {
    let query = fold(query.trim());
    fonts
        .iter()
        .filter(|font| category.is_none_or(|c| font.category == c))
        .filter(|font| query.is_empty() || fold(&font.family).contains(&query))
        .collect()
}

/// Nombre de pages pour `count` entrées (au moins une).
pub fn page_count(count: usize) -> usize {
    count.div_ceil(PAGE_SIZE).max(1)
}

/// Entrées de la page `page` (à partir de 0), bornée à la dernière.
pub fn page_of<T>(items: &[T], page: usize) -> &[T] {
    let last = page_count(items.len()) - 1;
    let start = page.min(last) * PAGE_SIZE;
    &items[start..(start + PAGE_SIZE).min(items.len())]
}

/// Catégories proposées dans le filtre.
pub const CATEGORIES: [&str; 5] = ["sans-serif", "serif", "monospace", "display", "handwriting"];

/// Adresse d'un fichier WOFF2 du CDN.
pub fn file_url(id: &str, subset: &str, weight: u16) -> String {
    format!("{CDN}/{id}@latest/{subset}-{weight}-normal.woff2")
}

/// Fichiers à télécharger : latin et latin-ext (s'il existe), en normal, puis en
/// gras (700) quand la police en a un.
pub fn files_to_fetch(font: &CatalogFont) -> Vec<(Part, String)> {
    let has = |subset: &str| font.subsets.iter().any(|s| s == subset);
    let regular_weight = if font.weights.contains(&400) {
        400
    } else {
        *font.weights.iter().min().unwrap_or(&400)
    };
    let mut files = vec![(Part::Latin, file_url(&font.id, "latin", regular_weight))];
    if has("latin-ext") {
        files.push((
            Part::LatinExt,
            file_url(&font.id, "latin-ext", regular_weight),
        ));
    }
    if font.weights.contains(&700) && regular_weight != 700 {
        files.push((Part::BoldLatin, file_url(&font.id, "latin", 700)));
        if has("latin-ext") {
            files.push((Part::BoldLatinExt, file_url(&font.id, "latin-ext", 700)));
        }
    }
    files
}

/// Liste enregistrée sur le disque, avec sa date de téléchargement.
#[derive(Serialize, Deserialize)]
struct CacheFile {
    fetched_at: u64,
    fonts: Vec<CatalogFont>,
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Relit la liste enregistrée : (polices, âge en secondes).
pub fn read_cache(path: &Path, now: u64) -> Option<(Vec<CatalogFont>, u64)> {
    let raw = std::fs::read(path).ok()?;
    if raw.len() > MAX_LIST_BYTES {
        return None;
    }
    let cache: CacheFile = serde_json::from_slice(&raw).ok()?;
    let fonts: Vec<CatalogFont> = cache
        .fonts
        .into_iter()
        .filter(|f| valid_font_id(&f.id) && LICENSES.contains(&f.license.as_str()))
        .collect();
    if fonts.is_empty() {
        return None;
    }
    Some((fonts, now.saturating_sub(cache.fetched_at)))
}

/// Enregistre la liste ; l'échec n'est pas grave, elle sera retéléchargée.
pub fn write_cache(path: &Path, fonts: &[CatalogFont], now: u64) {
    let cache = CacheFile {
        fetched_at: now,
        fonts: fonts.to_vec(),
    };
    let Ok(json) = serde_json::to_vec(&cache) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let part = path.with_extension("part");
    if std::fs::write(&part, json).is_ok() && std::fs::rename(&part, path).is_err() {
        let _ = std::fs::remove_file(&part);
    }
}

/// Réponse d'une requête : les octets, ou la raison de l'échec.
pub type Response = Result<Vec<u8>, String>;

/// Client HTTP, remplaçable dans les tests : le rappel est appelé depuis un autre fil.
pub trait FontHttp {
    fn get(&self, url: &str, max_bytes: usize, done: Box<dyn FnOnce(Response) + Send + 'static>);
}

/// Client réel, sur `ehttp`.
pub struct EhttpFonts;

impl FontHttp for EhttpFonts {
    fn get(&self, url: &str, max_bytes: usize, done: Box<dyn FnOnce(Response) + Send + 'static>) {
        ehttp::fetch(ehttp::Request::get(url), move |result| {
            done(match result {
                Ok(response) if response.ok && response.bytes.len() <= max_bytes => {
                    Ok(response.bytes)
                }
                Ok(response) if response.ok => Err("réponse trop grosse".into()),
                Ok(response) => Err(format!("HTTP {}", response.status)),
                Err(error) => Err(error),
            });
        });
    }
}

/// Client partagé entre l'application et ses fils de rappel.
pub type SharedHttp = Arc<dyn FontHttp>;

#[cfg(test)]
#[path = "../tests/test_services_fontsource.rs"]
mod tests;
