//! Les dix thèmes intégrés.
//!
//! Chaque thème est une graine de dix couleurs par mode, dont la palette
//! complète est dérivée. Classique garde les palettes d'origine, au bit près.
//! Les palettes de Salon à Encre s'inspirent de palettes publiques sous
//! licence MIT (cf. Crédits) ; leurs noms n'apparaissent pas dans l'interface.

// Branché par l'application des réglages (commit suivant).
#![allow(dead_code)]

use eframe::egui::Color32;

use super::colors::{palette_from_seed, Seed};
use super::i18n::{self, Entry};
use super::theme::{ResolvedTheme, Typography, DARK, LIGHT};

const fn c(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Graine au format `fond, surface, champ, bordure, texte, discret, accent, lien, erreur, succès`.
const fn seed(v: [u32; 10]) -> Seed {
    Seed {
        background: c(v[0]),
        surface: c(v[1]),
        input_bg: c(v[2]),
        border: c(v[3]),
        text: c(v[4]),
        text_muted: c(v[5]),
        accent: c(v[6]),
        link: c(v[7]),
        danger: c(v[8]),
        success: c(v[9]),
    }
}

/// Texte clair de Lisible sur fond sombre, un cran sous le blanc de Classique.
const READABLE_DARK_TEXT: Color32 = c(0xE8EBEF);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BuiltinTheme {
    Classic,
    Lounge,
    Frost,
    Nocturne,
    Sand,
    Slate,
    Marshmallow,
    Ink,
    Forest,
    Readable,
}

impl BuiltinTheme {
    /// Ordre d'affichage dans la galerie.
    pub(crate) const ALL: [Self; 10] = [
        Self::Classic,
        Self::Lounge,
        Self::Frost,
        Self::Nocturne,
        Self::Sand,
        Self::Slate,
        Self::Marshmallow,
        Self::Ink,
        Self::Forest,
        Self::Readable,
    ];

    /// Identifiant stable, écrit dans les préférences.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Lounge => "lounge",
            Self::Frost => "frost",
            Self::Nocturne => "nocturne",
            Self::Sand => "sand",
            Self::Slate => "slate",
            Self::Marshmallow => "marshmallow",
            Self::Ink => "ink",
            Self::Forest => "forest",
            Self::Readable => "readable",
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == id)
    }

    pub(crate) fn name(self) -> Entry {
        match self {
            Self::Classic => i18n::THEME_CLASSIQUE,
            Self::Lounge => i18n::THEME_SALON,
            Self::Frost => i18n::THEME_GIVRE,
            Self::Nocturne => i18n::THEME_NOCTURNE,
            Self::Sand => i18n::THEME_SABLE,
            Self::Slate => i18n::THEME_ARDOISE,
            Self::Marshmallow => i18n::THEME_GUIMAUVE,
            Self::Ink => i18n::THEME_ENCRE,
            Self::Forest => i18n::THEME_FORET,
            Self::Readable => i18n::THEME_LISIBLE,
        }
    }

    pub(crate) fn description(self) -> Entry {
        match self {
            Self::Classic => i18n::THEME_CLASSIQUE_DESCRIPTION,
            Self::Lounge => i18n::THEME_SALON_DESCRIPTION,
            Self::Frost => i18n::THEME_GIVRE_DESCRIPTION,
            Self::Nocturne => i18n::THEME_NOCTURNE_DESCRIPTION,
            Self::Sand => i18n::THEME_SABLE_DESCRIPTION,
            Self::Slate => i18n::THEME_ARDOISE_DESCRIPTION,
            Self::Marshmallow => i18n::THEME_GUIMAUVE_DESCRIPTION,
            Self::Ink => i18n::THEME_ENCRE_DESCRIPTION,
            Self::Forest => i18n::THEME_FORET_DESCRIPTION,
            Self::Readable => i18n::THEME_LISIBLE_DESCRIPTION,
        }
    }

    /// Graines sombre et claire des thèmes dérivés ; `None` pour Classique et Lisible.
    fn seeds(self) -> Option<(Seed, Seed)> {
        let (dark, light) = match self {
            Self::Classic | Self::Readable => return None,
            Self::Lounge => (
                [
                    0x313338, 0x2B2D31, 0x383A40, 0x3F4147, 0xDBDEE1, 0x949BA4, 0x5865F2, 0x00A8FC,
                    0xF23F43, 0x23A55A,
                ],
                [
                    0xFFFFFF, 0xF2F3F5, 0xEBEDEF, 0xE3E5E8, 0x313338, 0x5C5E66, 0x5865F2, 0x006CE7,
                    0xDA373C, 0x248046,
                ],
            ),
            Self::Frost => (
                [
                    0x2E3440, 0x3B4252, 0x434C5E, 0x4C566A, 0xECEFF4, 0xD8DEE9, 0x88C0D0, 0x81A1C1,
                    0xBF616A, 0xA3BE8C,
                ],
                [
                    0xECEFF4, 0xE5E9F0, 0xD8DEE9, 0xC2CAD8, 0x2E3440, 0x4C566A, 0x5E81AC, 0x5E81AC,
                    0xBF616A, 0xA3BE8C,
                ],
            ),
            Self::Nocturne => (
                [
                    0x282A36, 0x21222C, 0x44475A, 0x6272A4, 0xF8F8F2, 0x9EA8C7, 0xBD93F9, 0x8BE9FD,
                    0xFF5555, 0x50FA7B,
                ],
                [
                    0xFFFBEB, 0xF5F0DC, 0xEFEDDC, 0xCFCFDE, 0x1F1F1F, 0x6C664B, 0x644AC9, 0x036A96,
                    0xCB3A2A, 0x14710A,
                ],
            ),
            Self::Sand => (
                [
                    0x002B36, 0x073642, 0x073642, 0x586E75, 0x93A1A1, 0x839496, 0x268BD2, 0x268BD2,
                    0xDC322F, 0x859900,
                ],
                [
                    0xFDF6E3, 0xEEE8D5, 0xEEE8D5, 0x93A1A1, 0x586E75, 0x657B83, 0x268BD2, 0x268BD2,
                    0xDC322F, 0x859900,
                ],
            ),
            Self::Slate => (
                [
                    0x282C34, 0x21252B, 0x2C313A, 0x3E4451, 0xABB2BF, 0x7F848E, 0x61AFEF, 0x56B6C2,
                    0xE06C75, 0x98C379,
                ],
                [
                    0xFAFAFA, 0xF0F0F1, 0xE5E5E6, 0xD4D4D4, 0x383A42, 0x696C77, 0x4078F2, 0x0184BC,
                    0xE45649, 0x50A14F,
                ],
            ),
            Self::Marshmallow => (
                [
                    0x1E1E2E, 0x181825, 0x313244, 0x45475A, 0xCDD6F4, 0xA6ADC8, 0xCBA6F7, 0x89B4FA,
                    0xF38BA8, 0xA6E3A1,
                ],
                [
                    0xEFF1F5, 0xE6E9EF, 0xCCD0DA, 0xBCC0CC, 0x4C4F69, 0x6C6F85, 0x8839EF, 0x1E66F5,
                    0xD20F39, 0x40A02B,
                ],
            ),
            Self::Ink => (
                [
                    0x0D1117, 0x161B22, 0x0D1117, 0x30363D, 0xE6EDF3, 0x8D96A0, 0x2F81F7, 0x4493F8,
                    0xF85149, 0x3FB950,
                ],
                [
                    0xFFFFFF, 0xF6F8FA, 0xFFFFFF, 0xD0D7DE, 0x1F2328, 0x656D76, 0x0969DA, 0x0969DA,
                    0xCF222E, 0x1A7F37,
                ],
            ),
            Self::Forest => (
                [
                    0x1B2420, 0x222D28, 0x2A3631, 0x3A4A42, 0xE3EDE7, 0x9FB5AA, 0x5FBF8F, 0x7CC7E8,
                    0xE5736B, 0x7FD18B,
                ],
                [
                    0xF4F8F5, 0xE8F0EB, 0xDDE8E1, 0xC3D3C9, 0x1E2B25, 0x4F6359, 0x2E7D57, 0x1F6F99,
                    0xB83A32, 0x2E7D3A,
                ],
            ),
        };
        Some((seed(dark), seed(light)))
    }

    /// Réglages de texte propres au thème.
    pub(crate) fn typography(self) -> Typography {
        match self {
            Self::Lounge => Typography::LOUNGE,
            Self::Readable => Typography::READABLE,
            _ => Typography::CLASSIC,
        }
    }

    /// Thème résolu, tel que le thème le définit, avant tout réglage.
    pub(crate) fn resolve(self) -> ResolvedTheme {
        let (dark, light) = match (self, self.seeds()) {
            (_, Some((dark, light))) => (palette_from_seed(&dark), palette_from_seed(&light)),
            (Self::Readable, None) => {
                let mut dark = DARK;
                dark.text = READABLE_DARK_TEXT;
                (dark, LIGHT)
            }
            _ => (DARK, LIGHT),
        };
        ResolvedTheme {
            revision: 0,
            dark,
            light,
            typography: self.typography(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_ui_themes.rs"]
mod tests;
