# 07 — Développement

[English](en/07-development.md) · **Français** · [Documentation](README.fr.md)

## Mise en place

Installer Rust (version minimale déclarée dans `Cargo.toml` : **1.95**) et les [paquets Linux](06-installation.md#dépendances-système-sous-linux) si nécessaire.

```sh
git clone https://github.com/abnd-corp/Abcom.git
cd Abcom
git config core.hooksPath .githooks
cargo build --locked
```

La configuration des services optionnels est décrite dans le [guide d'installation](06-installation.md). Ne jamais versionner de secret.

Python 3 est nécessaire au contrôle documentaire, sans paquet Python supplémentaire. Il ne fait pas partie des dépendances d’exécution d’Abcom.

## Contrôles avant contribution

```sh
make check
# Documentation only / Documentation seule
make check-docs
```

`make check` exécute :

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
```

Pour vérifier aussi la configuration minimale :

```sh
cargo check --all-targets --no-default-features --locked
cargo test --no-default-features --locked
```

Les tests réseau ouvrent de vraies sockets locales : un bac à sable qui les interdit produit des échecs de permissions. Ne pas confondre cette restriction avec une régression réseau.

Dernière validation locale du 11 novembre 2026 sur macOS : **642 tests réussis** avec toutes les fonctionnalités, **640** sans fonctionnalités par défaut. Répartition : 634 ou 632 dans la bibliothèque, 2 dans le binaire, 1 scénario P2P et 5 tests d'interaction externes. Le test KSNI sur bus D-Bus privé est réservé à la CI Linux ; son exécution et le rendu sur un vrai bureau Linux restent à confirmer. Le [compte rendu](08-historique-et-audits.md) précise l'environnement et les contrôles non effectués. Ce relevé daté ne remplace pas une nouvelle exécution.

## Organisation des tests

Les tests unitaires vivent dans `src/tests/`, raccordés au module testé :

```rust
#[cfg(test)]
#[path = "../tests/test_app_groups.rs"]
mod tests;
```

Un nouveau module reçoit son fichier de tests. Quelques modules historiques gardent des tests inline ; les scénarios de `tests/` utilisent l'API publique. `bash scripts/integration_test.sh` rejoue uniquement l'échange P2P authentifié sur TCP. La découverte entre processus et les intégrations natives restent au [cahier de tests](10-cahier-de-tests.md).

## Contribution et workflow

Les règles de code et de commit sont dans [CONTRIBUTING.md](../CONTRIBUTING.md) ; la [charte Abnd](https://github.com/abnd-corp/gouvernance/blob/main/CHARTE.md) fixe le fonctionnement de l'organisation. Ces fichiers de gouvernance ne sont pas dupliqués ni réécrits dans la documentation.

- Travailler sur une branche `feature/…`, avec une issue et un jalon pour la PR selon la charte.
- Proposer les changements par PR ; suivre le flux de promotion de l'organisation.
- Conserver les commits `type(scope): description` en français, à l'impératif, sans majuscule après `:` ni point final. Une intention par commit.
- Les agents ne poussent jamais sans accord explicite et n'ajoutent aucun trailer `Co-Authored-By`. Un merge demande une instruction humaine explicite et les validations requises.

Pour la documentation : mettre à jour les deux langues et exécuter le contrôle de liens. Le [sommaire](README.fr.md#maintenir-ces-guides) décrit le contrat de traduction.

## Intégration continue

| Workflow | Rôle |
|---|---|
| [ci-dev.yml](../.github/workflows/ci-dev.yml) | Format, Clippy, release, tests Linux, MSRV, compilation minimale, checks macOS/Windows, couverture, audit |
| [ci-main.yml](../.github/workflows/ci-main.yml) | Contrôles de livraison et scénario P2P |
| [release.yml](../.github/workflows/release.yml) | Archives Linux x86-64, macOS arm64, Windows x86-64 et `SHA256SUMS.txt` sur tag `v*` ou lancement manuel |
| [dependencies.yml](../.github/workflows/dependencies.yml) | Veille des dépendances |

La MSRV vient de `rust-version` dans `Cargo.toml`, pas du nom « stable ». Les contrôles de compilation CI ne certifient pas le tray, les notifications, les sélecteurs natifs ou l'accessibilité sur les trois OS.

## Dépendances et sécurité

```sh
cargo audit
cargo deny --locked check licenses sources bans
cargo deny --locked check advisories
```

`Cargo.lock` est versionné. Justifier les nouvelles dépendances, limiter leurs fonctionnalités et ne pas monter leurs versions dans un correctif sans rapport. Le [relevé du 11 novembre](08-historique-et-audits.md) signale des avis RustSec non résolus ; les tests applicatifs verts ne rendent pas cet audit vert.

Le rendu utilise eframe/egui **0.37** avec **wgpu**. Ne pas réactiver les features par défaut d'eframe qui introduiraient le renderer Glow. Sur macOS, la chaîne actuelle ne doit pas relier Glow ; ailleurs wgpu peut tirer sa propre dépendance GLES de repli.

Le profil release utilise LTO thin, une unité de codegen, suppression des symboles et `panic = "abort"`. Les mesures de temps et de mémoire destinées à une publication doivent être faites en release et datées.

## Scripts utiles

| Script | Rôle |
|---|---|
| [run-multi.sh](../scripts/run-multi.sh) | Plusieurs instances locales |
| [seed-demo.py](../scripts/seed-demo.py) | Données de démonstration |
| [integration_test.sh](../scripts/integration_test.sh) | Test P2P headless |
| [check_docs.py](../scripts/check_docs.py) | Paires EN/FR, liens, ancres et identifiants de tests |
| [generate_branding.py](../scripts/generate_branding.py) | Sources SVG et liste des exports graphiques |
| [build-and-distribute.sh](../scripts/build-and-distribute.sh) | Archive de distribution locale |
| [install-windows.ps1](../scripts/install-windows.ps1) | Installation Windows |
| [abcom-install.sh](../scripts/abcom-install.sh) | Installation Linux depuis un binaire |
| [uninstall.sh](../scripts/uninstall.sh) | Désinstallation Linux |

## Ressources embarquées

Le code est sous [AGPL-3.0-or-later](../LICENSE). Les polices et emojis ont leurs propres licences, présentes dans les ressources et dans Paramètres → Crédits. Noto Sans, Noto Sans Symbols 2, Inter et les polices proposées sont sous OFL-1.1 ; GNU Unifont offre OFL-1.1 ou GPLv2+ avec exception d'embarquement. Consulter les notices de [assets/fonts](../assets/fonts/) pour les textes applicables.

Les glyphes de repli couvrent les écritures non latines ; les emojis des conversations utilisent les PNG embarqués. Vérifier les tests de couverture avant de modifier cette chaîne.

Le [pack graphique SVG/PNG](../assets/branding/README.fr.md) contient les symboles, logotypes et variantes d'icônes. Sa documentation décrit les exports reproductibles avec la dépendance SVG déjà présente et l'usage des masques du tray macOS.
