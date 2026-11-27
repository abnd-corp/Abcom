//! Notifications : réglages persistés et choix des signaux pour un événement entrant.

use std::collections::HashSet;

use crate::app::AppState;

/// Réglages de l'onglet Notifications, tous actifs par défaut (table kv).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NotificationSettings {
    /// Notification système quand Abcom n'est pas au premier plan.
    pub(crate) system: bool,
    /// Texte du message dans la notification système, sinon « Nouveau message ».
    pub(crate) preview: bool,
    pub(crate) sound: bool,
    /// Nombre de non-lus sur l'icône de l'application et pastille du tray.
    pub(crate) badge: bool,
    /// Clignotement dans la barre des tâches, rebond dans le Dock.
    pub(crate) attention: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            system: true,
            preview: true,
            sound: true,
            badge: true,
            attention: true,
        }
    }
}

impl NotificationSettings {
    const SYSTEM: &'static str = "notif_system";
    const PREVIEW: &'static str = "notif_preview";
    const SOUND: &'static str = "notif_sound";
    const BADGE: &'static str = "notif_badge";
    const ATTENTION: &'static str = "notif_attention";

    pub(crate) fn load(state: &AppState) -> Self {
        Self {
            system: state.pref_bool(Self::SYSTEM, true),
            preview: state.pref_bool(Self::PREVIEW, true),
            sound: state.pref_bool(Self::SOUND, true),
            badge: state.pref_bool(Self::BADGE, true),
            attention: state.pref_bool(Self::ATTENTION, true),
        }
    }

    pub(crate) fn save(&self, state: &mut AppState) {
        for (key, enabled) in [
            (Self::SYSTEM, self.system),
            (Self::PREVIEW, self.preview),
            (Self::SOUND, self.sound),
            (Self::BADGE, self.badge),
            (Self::ATTENTION, self.attention),
        ] {
            state.set_pref(key, if enabled { "1" } else { "0" });
        }
    }
}

/// Conversations en sourdine ; `None` désigne « Tous ».
pub(crate) type MutedConversations = HashSet<Option<String>>;

const MUTED_KEY: &str = "muted_conversations";

pub(crate) fn load_muted(state: &AppState) -> MutedConversations {
    state
        .pref_str(MUTED_KEY)
        .and_then(|json| serde_json::from_str::<Vec<Option<String>>>(json).ok())
        .map(|list| list.into_iter().collect())
        .unwrap_or_default()
}

pub(crate) fn save_muted(state: &mut AppState, muted: &MutedConversations) {
    // Ordre stable : la valeur stockée ne change pas d'une écriture à l'autre.
    let mut list: Vec<&Option<String>> = muted.iter().collect();
    list.sort();
    match serde_json::to_string(&list) {
        Ok(json) => state.set_pref(MUTED_KEY, &json),
        Err(error) => tracing::warn!(%error, "sourdines non enregistrées"),
    }
}

/// État de la fenêtre à l'arrivée d'un événement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WindowPresence {
    /// Fenêtre active : l'utilisateur regarde Abcom.
    Focused,
    /// Fenêtre ouverte mais derrière une autre application, ou réduite.
    Background,
    /// Fenêtre repliée dans le tray.
    Hidden,
}

/// Signaux à émettre pour un événement entrant.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Alert {
    pub(crate) system: bool,
    pub(crate) sound: bool,
    pub(crate) attention: bool,
}

/// Rien pour une conversation en sourdine ni pour celle qu'on regarde ; un son seul pour une
/// autre conversation de la fenêtre active ; sinon tout ce que les réglages permettent. Une
/// fenêtre repliée n'a pas de bouton à faire clignoter.
pub(crate) fn alert_for(
    settings: &NotificationSettings,
    presence: WindowPresence,
    conversation_open: bool,
    muted: bool,
) -> Alert {
    if muted {
        return Alert::default();
    }
    match presence {
        WindowPresence::Focused => Alert {
            sound: settings.sound && !conversation_open,
            ..Alert::default()
        },
        WindowPresence::Background => Alert {
            system: settings.system,
            sound: settings.sound,
            attention: settings.attention,
        },
        WindowPresence::Hidden => Alert {
            system: settings.system,
            sound: settings.sound,
            attention: false,
        },
    }
}

#[cfg(test)]
#[path = "../tests/test_ui_notifications.rs"]
mod tests;
