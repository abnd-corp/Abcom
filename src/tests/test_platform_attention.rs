use super::*;

#[test]
fn badge_label_hides_zero_and_caps_long_counts() {
    assert_eq!(badge_label(0), None);
    assert_eq!(badge_label(1).as_deref(), Some("1"));
    assert_eq!(badge_label(99).as_deref(), Some("99"));
    assert_eq!(badge_label(100).as_deref(), Some("99+"));
}

#[test]
fn badge_dot_is_an_opaque_disc_with_a_transparent_outside() {
    let size = 16;
    let pixels = badge_dot_rgba(size);
    assert_eq!(pixels.len(), (size * size * 4) as usize);
    let alpha = |x: u32, y: u32| pixels[((y * size + x) * 4 + 3) as usize];
    assert_eq!(alpha(8, 8), 255, "centre plein");
    assert_eq!(alpha(0, 0), 0, "coin transparent");
    assert!(alpha(0, 8) > 0 && alpha(0, 8) < 255, "bord lissé");
    // Rouge partout : seule l'opacité dessine le disque.
    assert!(pixels
        .as_chunks::<4>()
        .0
        .iter()
        .all(|p| p[..3] == [229, 72, 77]));
}
