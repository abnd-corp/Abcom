use std::sync::{Mutex, MutexGuard};

/// Verrouillage tolérant à l'empoisonnement : si un thread a paniqué en
/// tenant le verrou, on récupère la donnée plutôt que de propager la panique
/// (les mutations sont toutes courtes, l'état reste cohérent au grain où on
/// le lit).
pub trait MutexExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Dimensions au-delà desquelles une image reçue est refusée sans être décodée.
///
/// `image` plafonne l'allocation à 512 Mo mais ne borne pas les dimensions : un
/// fichier de quelques kilo-octets peut donc nous faire allouer beaucoup. On lit
/// l'en-tête d'abord, on décode seulement si c'est raisonnable.
pub const MAX_IMAGE_SIDE: u32 = 8192;

/// Décode une image reçue du réseau, en refusant les dimensions déraisonnables.
pub fn decode_image_bounded(bytes: &[u8]) -> Option<image::DynamicImage> {
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let (width, height) = reader.into_dimensions().ok()?;
    if width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE {
        tracing::warn!("image refusée : {width}x{height} dépasse {MAX_IMAGE_SIDE}");
        return None;
    }
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

/// Secondes écoulées depuis l'epoch Unix ; 0 si l'horloge système est avant 1970.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Taille lisible en unités décimales (ko, Mo, Go), partout la même dans l'interface.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [(&str, u64); 3] = [("Go", 1_000_000_000), ("Mo", 1_000_000), ("ko", 1_000)];
    for (unit, scale) in UNITS {
        if bytes >= scale {
            return format!("{:.1} {unit}", bytes as f64 / scale as f64);
        }
    }
    format!("{bytes} o")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_uses_decimal_units() {
        assert_eq!(human_bytes(999), "999 o");
        assert_eq!(human_bytes(1_000), "1.0 ko");
        assert_eq!(human_bytes(1_500_000), "1.5 Mo");
        assert_eq!(human_bytes(2_000_000_000), "2.0 Go");
    }

    #[test]
    fn lock_safe_recovers_after_poisoning() {
        let mutex = Mutex::new(0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("boom");
        }));
        assert!(result.is_err());
        assert!(mutex.is_poisoned());

        let guard = mutex.lock_safe();
        assert_eq!(*guard, 0);
    }
}
