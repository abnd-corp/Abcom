//! Polices en ligne : liste du catalogue et téléchargement, à la demande et avec accord.
//!
//! Tant que l'utilisateur n'a pas coché l'autorisation, rien n'est demandé au
//! réseau : ni à l'ouverture de l'onglet, ni au démarrage.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui;

use super::appearance_tab::FontZone;
use super::fonts::{FontChoice, ImportOutcome};
use super::{i18n, AbcomApp};
use crate::services::fontsource::{
    self, CatalogFont, EhttpFonts, Part, Response, SharedHttp, CACHE_TTL_SECS, LIST_URL,
    MAX_FILE_BYTES, MAX_LIST_BYTES,
};

/// Où en est la liste du catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListStatus {
    /// Pas encore demandée.
    Idle,
    Loading,
    Ready,
    Failed,
}

enum Event {
    List {
        generation: u64,
        result: Response,
    },
    Part {
        generation: u64,
        part: Part,
        result: Response,
    },
}

/// Téléchargement en cours d'une police.
pub(crate) struct Download {
    pub(crate) font: CatalogFont,
    zone: FontZone,
    expected: usize,
    received: Vec<(Part, Response)>,
}

pub(crate) struct OnlineFonts {
    pub(crate) fonts: Vec<CatalogFont>,
    /// La liste vient du disque et n'a pas pu être rafraîchie.
    pub(crate) stale: bool,
    pub(crate) status: ListStatus,
    pub(crate) query: String,
    pub(crate) category: Option<String>,
    pub(crate) page: usize,
    pub(crate) download: Option<Download>,
    /// Change à chaque annulation : les réponses d'une génération périmée sont ignorées.
    generation: u64,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    pub(crate) http: SharedHttp,
}

impl OnlineFonts {
    pub(crate) fn new(http: SharedHttp) -> Self {
        let (tx, rx) = channel();
        Self {
            fonts: Vec::new(),
            stale: false,
            status: ListStatus::Idle,
            query: String::new(),
            category: None,
            page: 0,
            download: None,
            generation: 0,
            tx,
            rx,
            http,
        }
    }

    pub(crate) fn real() -> Self {
        Self::new(Arc::new(EhttpFonts))
    }
}

impl AbcomApp {
    fn catalog_cache_path(&self) -> std::path::PathBuf {
        self.fonts.dir().join("fontsource.json")
    }

    /// Abandonne tout ce qui est en route : les réponses tardives seront ignorées.
    pub(crate) fn online_cancel(&mut self) {
        self.online.generation += 1;
        self.online.download = None;
        if self.online.status == ListStatus::Loading {
            self.online.status = if self.online.fonts.is_empty() {
                ListStatus::Idle
            } else {
                ListStatus::Ready
            };
        }
    }

    /// Charge la liste : le disque d'abord, le réseau si elle est absente ou ancienne.
    pub(crate) fn online_load_list(&mut self, ctx: &egui::Context) {
        if !self.appearance.online_fonts_allowed || self.online.status != ListStatus::Idle {
            return;
        }
        let cached = fontsource::read_cache(&self.catalog_cache_path(), fontsource::now_secs());
        match cached {
            Some((fonts, age)) if age < CACHE_TTL_SECS => {
                self.online.fonts = fonts;
                self.online.status = ListStatus::Ready;
                return;
            }
            Some((fonts, _)) => {
                self.online.fonts = fonts;
                self.online.stale = true;
            }
            None => {}
        }
        self.online.status = ListStatus::Loading;
        let (tx, ctx, generation) = (self.online.tx.clone(), ctx.clone(), self.online.generation);
        self.online.http.get(
            LIST_URL,
            MAX_LIST_BYTES,
            Box::new(move |result| {
                let _ = tx.send(Event::List { generation, result });
                ctx.request_repaint();
            }),
        );
    }

    /// Lance le téléchargement d'une police pour une zone. Un nouveau clic annule le précédent.
    pub(crate) fn online_download(&mut self, ctx: &egui::Context, id: &str, zone: FontZone) {
        if !self.appearance.online_fonts_allowed {
            return;
        }
        let Some(font) = self.online.fonts.iter().find(|f| f.id == id).cloned() else {
            return;
        };
        self.online.generation += 1;
        let files = fontsource::files_to_fetch(&font);
        self.online.download = Some(Download {
            font,
            zone,
            expected: files.len(),
            received: Vec::new(),
        });
        for (part, url) in files {
            let (tx, ctx, generation) =
                (self.online.tx.clone(), ctx.clone(), self.online.generation);
            self.online.http.get(
                &url,
                MAX_FILE_BYTES,
                Box::new(move |result| {
                    let _ = tx.send(Event::Part {
                        generation,
                        part,
                        result,
                    });
                    ctx.request_repaint();
                }),
            );
        }
    }

    /// Traite les réponses arrivées depuis les fils réseau.
    pub(crate) fn online_poll(&mut self) {
        while let Ok(event) = self.online.rx.try_recv() {
            match event {
                Event::List { generation, result } if generation == self.online.generation => {
                    self.online_list_arrived(result);
                }
                Event::Part {
                    generation,
                    part,
                    result,
                } if generation == self.online.generation => {
                    if let Some(download) = self.online.download.as_mut() {
                        download.received.push((part, result));
                        if download.received.len() >= download.expected {
                            self.online_finish_download();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn online_list_arrived(&mut self, result: Response) {
        match result.and_then(|bytes| fontsource::parse_catalog(&bytes)) {
            Ok(fonts) => {
                fontsource::write_cache(&self.catalog_cache_path(), &fonts, fontsource::now_secs());
                self.online.fonts = fonts;
                self.online.stale = false;
                self.online.status = ListStatus::Ready;
            }
            Err(error) => {
                tracing::warn!("liste de polices en ligne : {error}");
                if self.online.fonts.is_empty() {
                    self.online.status = ListStatus::Failed;
                    self.notify(self.t(i18n::LISTE_INDISPONIBLE));
                } else {
                    self.online.stale = true;
                    self.online.status = ListStatus::Ready;
                }
            }
        }
    }

    fn online_finish_download(&mut self) {
        let Some(download) = self.online.download.take() else {
            return;
        };
        let mut regular: Vec<(Part, Vec<u8>)> = Vec::new();
        let mut bold: Vec<(Part, Vec<u8>)> = Vec::new();
        for (part, result) in download.received {
            match result {
                Ok(bytes) if part.is_bold() => bold.push((part, bytes)),
                Ok(bytes) => regular.push((part, bytes)),
                Err(error) if part.is_required() => {
                    tracing::warn!("{} : {error}", download.font.id);
                    self.notify(self.t(i18n::TELECHARGEMENT_ECHOUE));
                    return;
                }
                // Un sous-ensemble ou un gras absent n'empêche pas d'utiliser la police.
                Err(error) => tracing::debug!("{} : {error}", download.font.id),
            }
        }
        // Le latin passe avant son extension, comme dans le catalogue.
        regular.sort_by_key(|(part, _)| *part as u8);
        bold.sort_by_key(|(part, _)| *part as u8);
        let regular: Vec<&[u8]> = regular.iter().map(|(_, b)| b.as_slice()).collect();
        let bold: Vec<&[u8]> = bold.iter().map(|(_, b)| b.as_slice()).collect();
        let source = format!("fontsource:{}", download.font.id);
        match self.fonts.import_set(&regular, &bold, &source) {
            Ok(ImportOutcome::Added(font)) | Ok(ImportOutcome::Existing(font)) => {
                *download.zone.choice_mut(&mut self.appearance) = FontChoice::Imported(font.id);
                let message = format!(
                    "{} {}",
                    self.t(i18n::POLICE_TELECHARGEE),
                    download.font.family
                );
                self.notify(&message);
            }
            Err(error) => {
                tracing::warn!("{} : {error:?}", download.font.id);
                self.notify(self.t(i18n::TELECHARGEMENT_ECHOUE));
            }
        }
    }
}
