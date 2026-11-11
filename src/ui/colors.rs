//! Calculs de couleur : contraste WCAG et palette dérivée d'une graine.

use eframe::egui::Color32;

use super::theme::Palette;

/// Seuil WCAG AA pour du texte courant.
pub(crate) const MIN_TEXT_CONTRAST: f32 = 4.5;
/// Seuil WCAG AA pour un élément graphique ou un grand texte.
pub(crate) const MIN_GRAPHIC_CONTRAST: f32 = 3.0;

/// Les dix couleurs qui définissent un thème ; le reste de la palette en dérive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Seed {
    pub(crate) background: Color32,
    pub(crate) surface: Color32,
    pub(crate) input_bg: Color32,
    pub(crate) border: Color32,
    pub(crate) text: Color32,
    pub(crate) text_muted: Color32,
    pub(crate) accent: Color32,
    pub(crate) link: Color32,
    pub(crate) danger: Color32,
    pub(crate) success: Color32,
}

fn linear(channel: u8) -> f32 {
    let c = f32::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Luminance relative WCAG 2.x, entre 0 (noir) et 1 (blanc).
pub(crate) fn luminance(color: Color32) -> f32 {
    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
}

/// Rapport de contraste WCAG 2.x entre deux couleurs, de 1 à 21.
pub(crate) fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

/// Mélange `a` vers `b` d'une part `t` (0 = `a`, 1 = `b`), opaque.
pub(crate) fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

/// Rapproche `fg` du noir ou du blanc, par pas de 2 %, jusqu'à atteindre `min`
/// sur `bg`. Une couleur déjà conforme est rendue telle quelle. Quand même le
/// noir ou le blanc ne suffit pas, c'est celui des deux qui contraste le plus.
pub(crate) fn ensure_contrast(fg: Color32, bg: Color32, min: f32) -> Color32 {
    if contrast_ratio(fg, bg) >= min {
        return fg;
    }
    let target = if contrast_ratio(Color32::WHITE, bg) >= contrast_ratio(Color32::BLACK, bg) {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    for step in 1..=50 {
        let candidate = mix(fg, target, step as f32 * 0.02);
        if contrast_ratio(candidate, bg) >= min {
            return candidate;
        }
    }
    target
}

/// Palette complète d'un mode, dérivée de la graine, avec les contrastes
/// minimaux garantis sur le fond.
pub(crate) fn palette_from_seed(seed: &Seed) -> Palette {
    let Seed {
        background,
        surface,
        input_bg,
        border,
        text,
        text_muted,
        accent,
        link,
        danger,
        success,
    } = *seed;
    let on_background = |color| ensure_contrast(color, background, MIN_TEXT_CONTRAST);
    // Le texte doit tenir sur le fond comme sur les cartes.
    let text = ensure_contrast(on_background(text), surface, MIN_TEXT_CONTRAST);
    let text_muted = ensure_contrast(on_background(text_muted), surface, MIN_TEXT_CONTRAST);
    let link = on_background(link);
    let accent = ensure_contrast(accent, background, MIN_GRAPHIC_CONTRAST);
    // La pastille teintée recule vers le fond tant que le texte n'y tient pas.
    let chip_mine = [0.30, 0.24, 0.18, 0.12, 0.06]
        .into_iter()
        .map(|share| mix(background, accent, share))
        .find(|chip| contrast_ratio(text, *chip) >= MIN_TEXT_CONTRAST)
        .unwrap_or(background);
    let tint_alpha = if luminance(background) < 0.5 { 7 } else { 10 };
    Palette {
        surface,
        surface_strong: mix(surface, text, 0.10),
        surface_hover: mix(surface, text, 0.07),
        separator: border,
        text,
        text_muted,
        accent,
        accent_soft: mix(accent, text, 0.15),
        danger: on_background(danger),
        success: on_background(success),
        receipt_read: link,
        link,
        block_bg: mix(background, text, 0.06),
        block_stripe: mix(background, text, 0.03),
        block_border: border,
        block_text: text,
        background,
        input_bg,
        chip: mix(background, text, 0.10),
        chip_mine,
        chip_mine_border: accent,
        row_tint: Color32::from_rgba_unmultiplied(text.r(), text.g(), text.b(), tint_alpha),
    }
}

#[cfg(test)]
#[path = "../tests/test_ui_colors.rs"]
mod tests;
