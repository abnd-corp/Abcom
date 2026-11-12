# 08 — Journal de maintenance et audits

[English](en/08-history-and-audits.md) · **Français** · [Documentation](README.fr.md)

Les résultats ci-dessous sont datés ; ils ne certifient pas toutes les versions ni tous les OS. Les procédures actives sont dans le [protocole de mesure](12-performances.md) et le [cahier de tests](10-cahier-de-tests.md).

## Repères historiques

- Avril 2026 : première messagerie LAN, JSON en clair et historique réécrit en fichiers.
- Juin 2026 : séparation des modules, tests sur sockets réelles et CI ; 164 tests après l'audit technique.
- Juillet 2026 : caches de rendu, SQLite WAL, transport Noise persistant, mode résident et groupes. Mesures macOS de l'époque : environ 0,2 % CPU, 155 Mo RSS et 8 threads après optimisation ; ce ne sont pas des budgets garantis aujourd'hui.
- Août 2026 : identifiants de groupes immuables, protocole 3, accusés après commit et file privée durable. Retrait du quota automatique après un incident de suppression : la purge est désormais manuelle.

Les anciens plans d'exécution et documents remplacés ont été retirés de l'arborescence courante. Les décisions toujours applicables figurent dans la [présentation](01-presentation.md#décisions-fondatrices) et l'[architecture](02-architecture.md).

## 12 novembre 2026 — Distribution locale Homebrew, WinGet et Nix

L’installation ne dépend plus d’une soumission aux catalogues publics : recettes générées depuis les artefacts réels et leurs SHA-256, cask dans un tap local, installateur Windows par utilisateur et flake Nix verrouillé. Le workflow de release prépare ces fichiers sans publier dans Homebrew/core, winget-pkgs ou nixpkgs.

Sur le Mac de test, l’ancien exécutable, le Launch Agent pointant vers un dossier disparu et deux bundles de diagnostic ont été sauvegardés puis retirés. Les dossiers de données de l’instance principale et des instances d’essai ont été conservés. Homebrew a réellement installé, réinstallé, désinstallé puis réinstallé Abcom ; les métadonnées des 74 fichiers de données contrôlés sont restées identiques, sans lecture de leur contenu. Une fenêtre native de l’application installée a été observée, puis l’arrêt normal a été vérifié.

Le premier essai de lancement était refusé. Le bundle possédait d’abord une signature incomplète ; le script produit désormais un sceau ad hoc complet et vérifié, après retrait des seules métadonnées Finder incompatibles. Gatekeeper refusait ensuite le téléchargement marqué par Homebrew, faute de Developer ID/notarisation. Le mode explicite `--local-build`, limité à un artefact local, retire la quarantaine seulement sur l’application vérifiée. Il ne change pas Gatekeeper globalement et n’est pas appliqué aux URL de release distantes. Une version téléchargée sans autorisation utilisateur ou notarisation peut toujours rester bloquée : voir le [guide des paquets](../packaging/README.fr.md).

Le paquet Nix a été compilé sur aarch64-darwin, sa signature de bundle vérifiée, puis ajouté et retiré d’un profil temporaire. SQLite était absent à l’édition de liens et a été déclaré explicitement. La recette Linux s’évalue ; son rendu graphique reste à tester sur Linux. Un manifeste produit avec une fixture passe le schéma officiel Microsoft 1.10.0 ; ce contrôle n’installe aucun binaire. Les outils d’installation Windows et leurs tests CI sont préparés mais n’ont pas été exécutés nativement sur ce Mac ; la validation WinGet sur Windows reste ouverte dans l’issue #68.

Validation locale : `make check` passe avec 659 tests Rust, Clippy strict et formatage ; les quatre tests Python des recettes passent. La compilation minimale et les liens documentaires EN/FR passent. Le contexte Docker exclut désormais les secrets et les données locales et inclut le script de build, la licence et les exemples. Buildx valide le Dockerfile sans avertissement ; ce contrôle ne vaut ni compilation du binaire en conteneur ni exécution graphique Linux.

## 12 novembre 2026 — Contrôle final des icônes et survol du bouton GIF

Le test du widget GIF reproduit une largeur passant de 33 à 31 points au survol et un décalage du texte d’un point. La bordure explicite était appliquée après la compensation des marges d’egui 0.37. Le bouton utilise désormais la même bordure dans ce calcul pour tous ses états. Le test compare le cadre et le texte sur 24 frames, au centre et près du bord, en clair/sombre, à 1×, 1,5× et 2×, sélecteur ouvert ou fermé. L’ancien code échoue ; le correctif passe. Suivi : [#64](https://github.com/abnd-corp/Abcom/issues/64).

Contrôles relancés : 650 tests réussis, formatage, Clippy strict, documentation EN/FR, compilation minimale, Rust 1.95 et build release. Les diagnostics natifs retrouvent un tray de 18 points dans les trois états et une largeur visible du Dock de 80,7 % hors bundle, 81,2 % dans le bundle. Les 124 exports, les 15 représentations ICO, la structure ICNS et le lanceur Linux sont contrôlés ; l’exécution graphique Windows/Linux reste à vérifier sur ces systèmes.

Deux cartes de partage EN/FR de 1280 × 640 px reprennent le nouveau logo. Le générateur contrôle leurs dimensions, leur opacité et leur poids inférieur à 1 Mo. Elles sont inspectées visuellement et accompagnées de la procédure d’activation GitHub dans le [guide graphique](../assets/branding/README.fr.md#aperçu-de-partage-github). Leur présence dans le dépôt ne modifie pas le réglage Social preview sur GitHub.

## 12 novembre 2026 — Icône du Dock après le rendu eframe

Les nouvelles captures montrent que corriger les fichiers ICNS ne suffisait pas. eframe 0.37 réappliquait l’icône de fenêtre sans marges lors de `pre_update`, après le réglage AppKit de démarrage. Le contrôle précédent validait les ressources et le tray, pas cette étape du cycle de vie ; son résultat ne prouvait donc pas la taille du Dock affiché.

Handy déclare son ICNS dans le bundle et conserve cette ressource lors du retour en politique Regular. Abcom suit désormais cette approche : l’icône de fenêtre macOS est explicitement vide pour désactiver le remplacement eframe ; le bundle récupère son ICNS d’origine. Un binaire lancé avec `cargo run` reçoit le même ICNS embarqué en repli. Windows/Linux conservent leur icône de fenêtre couleur.

Le nouveau diagnostic `check_macos_dock_after_frame` lance eframe, attend trois frames puis lit et exporte l’icône réelle de `NSApplication`. L’option `--reproduce-old` réactive volontairement le chemin fautif et doit échouer ; le mode normal doit réussir, y compris depuis un bundle `.app`. Les relevés de ce diagnostic remplacent la seule inspection des PNG pour le contrôle du Dock.

Résultats : ancien chemin refusé avec 97,3 % de largeur visible ; chemin corrigé accepté à 80,7 % depuis le binaire nu et 81,2 % depuis le bundle. Les images proviennent de l’icône native après trois frames eframe. Barrière locale : 649 tests réussis, Clippy strict, compilation minimale, Rust 1.95 et build release au vert.

## 12 novembre 2026 — Correction des tailles natives

Les captures utilisateur ont révélé un tray et un Dock surdimensionnés malgré des bitmaps valides. Le backend tray-icon traite le PNG comme une image native plafonnée à 22 points ; le recadrage accentuait le problème. Correction : image AppKit de 18 points, bitmap de 36 px et emplacement carré système, réappliqués après chaque badge. Le diagnostic natif mesure 18 × 18 pt / 36 × 36 px et un bouton de 22 × 22 pt sur ce Mac, dans les trois états normal/non lu/lu.

Le corps de l’ICNS est ramené à 824/1024, uniquement pour macOS. Les icônes Apple Réglages et Terminal et le nouvel export ont une largeur visible de 206/256 à alpha > 128. Windows reçoit 15 tailles ICO et un tray dimensionné d’après ses métriques DPI ; ses types d’API sont vérifiés avec les bindings 0.61.2, sans exécution Windows. Linux conserve le dimensionnement par son panneau SNI. La vérification visuelle des autres bureaux reste nécessaire.

## 12 novembre 2026 — Logos natifs

Le logo couleur est embarqué dans la fenêtre et les lanceurs. macOS utilise un Dock haute définition et un masque de barre de menus ; le masque est conservé lors des bascules du badge non lu. Windows reçoit un ICO multi-résolution intégré par `winresource`, dépendance de compilation uniquement. Les installateurs Linux posent le PNG dans hicolor. Le script de packaging macOS et le workflow de release produisent un bundle `.app` avec ICNS et métadonnées.

Validation locale macOS : 645 tests réussis, Clippy strict, compilation sans fonctionnalités par défaut, Rust 1.95, release et audit complet passent. Le test natif AppKit a décodé l'ICNS et créé le tray template avec bascule non lu/lu ; aucun échec ni erreur de mise à jour. `iconutil` relit dix représentations standard/Retina ; les six entrées ICO et les métadonnées du bundle sont contrôlées. Le rendu réel clair/sombre et Retina, ainsi que l'intégration Windows/Linux, restent à vérifier dans L9/M7/W7. Cette vérification locale ne remplace pas la CI des autres plateformes.

## 11 novembre 2026 — Tray KSNI et exceptions suivies

Le backend AppIndicator/GTK est remplacé par KSNI dans tray-icon 0.26.1, avec ses fonctionnalités par défaut désactivées. GTK, libappindicator et proc-macro-error ont disparu du lockfile. Le thread GTK et son réveil toutes les 200 ms sont supprimés ; les menus et le badge passent par le worker KSNI. Le contournement des variables GTK héritées de Snap n'est plus utile et a été retiré.

Trois exceptions nominatives, approuvées, visent uniquement RUSTSEC-2026-0370, 0371 et 0372. Leur motif et l'[issue #59](https://github.com/abnd-corp/Abcom/issues/59) figurent dans `deny.toml`. Cette issue reste ouverte jusqu'au retrait de toutes les exceptions. La CI vérifie cet invariant et exécute désormais l'audit complet ; fermer l'issue alors qu'une exception subsiste fait échouer le contrôle. Revue au 30 novembre 2026, avec justification et déplacement du jalon si nécessaire. Aucun avis de vulnérabilité n'est exempté.

Un test Linux explicite crée le tray et change son badge sur un bus D-Bus privé, sans GTK ; la CI le lance avec `dbus-run-session`. Ce test ne valide pas l'affichage réel du panneau, qui reste au cahier de tests. Les dépendances système GTK/AppIndicator ont été retirées de la CI et de l'image de test.

Validation locale sur macOS : 642 tests toutes fonctionnalités et 640 sans fonctionnalités par défaut, aucun échec. Formatage, Clippy strict, compilation release, contrôle documentaire EN/FR, compilation avec Rust 1.95.0 et audit complet `cargo deny --locked check` passent. Le test D-Bus Linux est préparé mais n'a pas été exécuté localement ; une session de bureau Linux réelle reste à vérifier. Le garde-fou de suivi a été contrôlé avec l'issue ouverte et un état fermé simulé, qui échoue comme prévu.

## 11 novembre 2026 — Mise à jour des dépendances

Toutes les dépendances directes ont été relevées vers leurs dernières versions stables compatibles, et le lockfile a été résolu à nouveau. Principales montées : eframe/egui 0.37, rusqlite 0.42, ZIP 9.5.1, rfd 0.18, tray-icon 0.26.1, resvg 0.50, rodio 0.23 et skrifa 0.50.

À cette étape, le backend AppIndicator sélectionné utilisait GTK 0.18.2 ; GTK 0.19 créait un conflit de liaison native. Le passage ultérieur à KSNI, décrit ci-dessus, retire cette chaîne. Les polices d'emoji monochromes d'egui sont activées explicitement pour conserver les glyphes précédemment fournis. Les API d'identifiants, rôles d'accessibilité et compensation du défilement ont été adaptées, ainsi que la lecture des noms ZIP. Le champ de durée de purge a maintenant un libellé accessible.

L'audit RustSec rafraîchi ne signale plus de vulnérabilité ni d'avis de sûreté des entrées-sorties sur le graphe résolu. **Il reste en échec sur quatre avis de dépendances non maintenues**, sans exception ajoutée à `deny.toml` :

| Dépendance | Chemin restant | Avis |
|---|---|---|
| `proc-macro-error 1.0.4` | GTK 0.18 et tray Linux | RUSTSEC-2024-0370 |
| `zbus_names 4.3.5` | AccessKit/AT-SPI et accessibilité Linux | RUSTSEC-2026-0371 |
| `zvariant 5.15.1` | AccessKit/AT-SPI et accessibilité Linux | RUSTSEC-2026-0370 |
| `zvariant_derive 5.15.1` | Même chaîne | RUSTSEC-2026-0372 |

Les dernières dépendances amont utilisées conservent ces composants. Lever ces quatre avis demande une évolution amont ou une adaptation dédiée ; désactiver le tray ou l'accessibilité pour masquer l'audit modifierait le produit. Les chiffres de l'audit précédent, ci-dessous, décrivent l'état antérieur à cette mise à jour.

### Validation de la mise à jour

| Contrôle | Résultat |
|---|---|
| Clippy toutes cibles et fonctionnalités, `-D warnings` | OK |
| Tests toutes fonctionnalités | 639 réussis, aucun échec |
| Tests sans fonctionnalités par défaut | 639 réussis, aucun échec |
| Compilation release | OK |
| Formatage et contrôle documentaire EN/FR | OK |
| Compilation toutes cibles et fonctionnalités avec Rust 1.95.0 | OK ; MSRV conservée |
| Licences, sources et interdictions | OK |
| Audit RustSec complet | Échec limité aux quatre avis de maintenance ci-dessus |

## 11 novembre 2026 — Revue des paramètres après Apparence

Reprise locale de `feature/apparence` au commit `b05a022`, sans push. Comparaison
avec les paramètres de `dev` : les grilles n'avaient pas de largeur maximale et
la licence imposait déjà 12 points. Le retour à la ligne global ajouté avec
Apparence réduisait les premières colonnes des grilles à leur minimum egui de
40 points. Stockage n'était testé qu'avant la réponse du calcul disque ; les
anciens contrôles de géométrie ignoraient les textes non interactifs.

Corrections : largeur des colonnes bornée dans Général, Stockage et Crédits,
actions de stockage repliables, un seul défilement par onglet et un état de
position distinct par onglet. La licence utilise la taille générale et des
paragraphes repliés à la largeur disponible ; un test vérifie la conservation
de tous ses mots. Le fichier `LICENSE` reste intact.

Six tests supplémentaires vérifient les textes et contrôles en FR/EN, fenêtres
560 × 360 et 860 × 600, tailles 100 et 160 %, la lisibilité des colonnes, la
licence et l'accès aux réglages de purge avec enregistrement. La matrice
existante utilise désormais un tableau de stockage rempli, avec des tailles en
Go et des milliers de fichiers. Captures GPU hors écran inspectées en taille
normale et petite fenêtre ; ce contrôle ne remplace pas une fenêtre native,
VoiceOver/NVDA ou des essais sur Windows/Linux.

### Contrôles exécutés sur macOS (Rust 1.99.1)

| Contrôle | Résultat |
|---|---|
| `cargo fmt --all --check` et `git diff --check` | OK |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | OK |
| `cargo test --all-features --locked` | 639 réussis : 631 bibliothèque, 2 binaire, 1 P2P, 5 interactions UI |
| `cargo check --all-targets --no-default-features --locked` | OK |
| `cargo test --no-default-features --locked` | 639 réussis, aucun échec |
| `cargo build --release --locked` | OK |
| `cargo deny --locked --offline check licenses sources bans` | OK, avertissements de doublons non bloquants |
| `cargo deny --locked check advisories` | Échec, détails ci-dessous |

Le premier lancement des tests réseau dans le bac à sable échouait à ouvrir
les sockets (`Operation not permitted`) ; la suite complète a ensuite été
rejouée avec l'accès local nécessaire et passe. Les captures GPU ont également
nécessité cet accès. Aucun test natif Windows/Linux ni contrôle avec la MSRV
1.95 n'a été exécuté ici (toolchain absente). Aucun lancement dans une vraie
fenêtre, sélecteur natif ou lecteur d'écran : ces essais restent au cahier de
tests, complété par les cas 12.30 à 12.32.

### Dépendances : audit bloquant

`cargo deny --locked check advisories`, avec la base RustSec mise à jour
(révision `fd6f39814c4d3c359fdbe223d0280dec4933f78f`), échoue sur **16 avis** :
11 vulnérabilités, un avis de sûreté des entrées-sorties et quatre paquets non maintenus.
`clipboard-win 5.4.1`, retiré du registre, produit en plus un avertissement.
Le problème ne se limite donc pas à ce dernier paquet. Les versions concernées étaient déjà présentes au début de cette reprise ; le correctif des paramètres ne modifiait ni les dépendances ni le lockfile. Ce relevé ne constitue pas une analyse de
l'exploitabilité de chaque avis dans Abcom. La mise à niveau des dépendances
reste à traiter séparément.

| Dépendance verrouillée | Catégorie cargo-deny | Avis |
|---|---|---|
| `libsqlite3-sys 0.38.2` | vulnerability | [RUSTSEC-2026-0459](https://rustsec.org/advisories/RUSTSEC-2026-0459) |
| `proc-macro-error 1.0.4` | unmaintained | [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348) |
| `rustls 0.23.43` | vulnerability | [RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349) |
| `rustls-webpki 0.103.13` | vulnerability | [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) |
| `tokio 1.53.1` | unsound | [RUSTSEC-2026-0386](https://rustsec.org/advisories/RUSTSEC-2026-0386) |
| `webpki-roots 1.0.9` | vulnerability | [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381) |
| `zbus_names 4.3.4` | unmaintained | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369) |
| `zip 8.6.0` | vulnerability | [RUSTSEC-2026-0429](https://rustsec.org/advisories/RUSTSEC-2026-0429) |
| `zvariant 5.13.1` | unmaintained | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370) |
| `zvariant_derive 5.13.1` | unmaintained | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372) |
