//! Signatures minisign des artefacts de release : Ed25519 sur une empreinte BLAKE2b-512 du
//! fichier, plus un commentaire de confiance signé qui lie le fichier à sa version.

use std::io::Read;

use base64::Engine as _;
use blake2::{Blake2b512, Digest};
use ed25519_dalek::{Signature, VerifyingKey};

/// Clés publiques acceptées : celle de la CI de release, puis la clé de secours hors ligne.
pub(crate) const TRUSTED_KEYS: [&str; 2] = [
    include_str!("../../assets/update/release.pub"),
    include_str!("../../assets/update/backup.pub"),
];

const KEY_ALGORITHM: &[u8; 2] = b"Ed";
/// Seules les signatures sur empreinte sont acceptées (l'ancien format signe le fichier entier).
const PREHASHED_ALGORITHM: &[u8; 2] = b"ED";
const TRUSTED_PREFIX: &str = "trusted comment: ";

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum SignatureError {
    Malformed(&'static str),
    UnknownKey,
    Invalid,
    Io(String),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(what) => write!(f, "signature illisible : {what}"),
            Self::UnknownKey => f.write_str("signature d'une clé inconnue"),
            Self::Invalid => f.write_str("signature invalide"),
            Self::Io(error) => write!(f, "lecture impossible : {error}"),
        }
    }
}

/// Clé publique minisign : identifiant de 8 octets et clé Ed25519.
pub(crate) struct PublicKey {
    id: [u8; 8],
    key: VerifyingKey,
}

impl PublicKey {
    /// Fichier `.pub` de minisign : la ligne de commentaire est ignorée.
    pub(crate) fn parse(text: &str) -> Result<Self, SignatureError> {
        let line = text
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with("untrusted comment:"))
            .ok_or(SignatureError::Malformed("clé publique vide"))?;
        let bytes = decode(line)?;
        let (algorithm, rest) = bytes
            .split_first_chunk::<2>()
            .ok_or(SignatureError::Malformed("clé publique courte"))?;
        if algorithm != KEY_ALGORITHM || rest.len() != 40 {
            return Err(SignatureError::Malformed("clé publique"));
        }
        let (id, key) = rest.split_at(8);
        let key: [u8; 32] = key.try_into().expect("longueur vérifiée");
        Ok(Self {
            id: id.try_into().expect("longueur vérifiée"),
            key: VerifyingKey::from_bytes(&key)
                .map_err(|_| SignatureError::Malformed("point Ed25519"))?,
        })
    }
}

/// Fichier `.minisig` décodé.
pub(crate) struct SignatureFile {
    key_id: [u8; 8],
    signature: Signature,
    trusted_comment: String,
    global_signature: Signature,
}

impl SignatureFile {
    pub(crate) fn parse(text: &str) -> Result<Self, SignatureError> {
        let mut lines = text.lines().map(str::trim_end);
        let _untrusted = lines.next();
        let signature = decode(lines.next().ok_or(SignatureError::Malformed("signature"))?)?;
        let trusted_comment = lines
            .next()
            .and_then(|line| line.strip_prefix(TRUSTED_PREFIX))
            .ok_or(SignatureError::Malformed("commentaire de confiance"))?
            .to_string();
        let global = decode(lines.next().ok_or(SignatureError::Malformed("signature globale"))?)?;

        let (algorithm, rest) = signature
            .split_first_chunk::<2>()
            .ok_or(SignatureError::Malformed("signature courte"))?;
        if algorithm != PREHASHED_ALGORITHM {
            return Err(SignatureError::Malformed("algorithme non pris en charge"));
        }
        let (key_id, signature) = rest
            .split_first_chunk::<8>()
            .ok_or(SignatureError::Malformed("signature courte"))?;
        Ok(Self {
            key_id: *key_id,
            signature: to_signature(signature)?,
            trusted_comment,
            global_signature: to_signature(&global)?,
        })
    }

    /// Commentaire de confiance ; ne fait foi qu'après [`Self::verify`].
    pub(crate) fn trusted_comment(&self) -> &str {
        &self.trusted_comment
    }

    /// Vérifie l'empreinte du contenu, puis le commentaire, avec l'une des clés données.
    pub(crate) fn verify(
        &self,
        keys: &[PublicKey],
        content: impl Read,
    ) -> Result<(), SignatureError> {
        let key = keys
            .iter()
            .find(|key| key.id == self.key_id)
            .ok_or(SignatureError::UnknownKey)?;
        let digest = blake2b(content)?;
        key.key
            .verify_strict(&digest, &self.signature)
            .map_err(|_| SignatureError::Invalid)?;
        let mut signed_comment = self.signature.to_bytes().to_vec();
        signed_comment.extend_from_slice(self.trusted_comment.as_bytes());
        key.key
            .verify_strict(&signed_comment, &self.global_signature)
            .map_err(|_| SignatureError::Invalid)
    }
}

/// Clés de confiance intégrées au binaire ; une clé illisible est un défaut de build.
pub(crate) fn trusted_keys() -> Vec<PublicKey> {
    TRUSTED_KEYS
        .iter()
        .filter_map(|text| match PublicKey::parse(text) {
            Ok(key) => Some(key),
            Err(error) => {
                tracing::error!(%error, "clé publique de mise à jour illisible");
                None
            }
        })
        .collect()
}

fn decode(line: &str) -> Result<Vec<u8>, SignatureError> {
    base64::engine::general_purpose::STANDARD
        .decode(line.trim())
        .map_err(|_| SignatureError::Malformed("base64"))
}

fn to_signature(bytes: &[u8]) -> Result<Signature, SignatureError> {
    let bytes: &[u8; 64] = bytes
        .try_into()
        .map_err(|_| SignatureError::Malformed("taille de signature"))?;
    Ok(Signature::from_bytes(bytes))
}

/// Empreinte en flux : un artefact de plusieurs dizaines de Mo ne passe pas en mémoire.
fn blake2b(mut content: impl Read) -> Result<Vec<u8>, SignatureError> {
    let mut hasher = Blake2b512::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = content
            .read(&mut buffer)
            .map_err(|error| SignatureError::Io(error.to_string()))?;
        if read == 0 {
            return Ok(hasher.finalize().to_vec());
        }
        hasher.update(&buffer[..read]);
    }
}

#[cfg(test)]
#[path = "../tests/test_update_signature.rs"]
mod tests;
