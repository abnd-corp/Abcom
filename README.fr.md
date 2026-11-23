[English](README.md) · **Français**

# Abcom — messagerie chiffrée sur réseau local

[![CI](https://github.com/abnd-corp/Abcom/actions/workflows/ci-dev.yml/badge.svg?branch=dev)](https://github.com/abnd-corp/Abcom/actions/workflows/ci-dev.yml)
[![Licence : AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue.svg)](LICENSE)

Abcom relie les ordinateurs d'un même réseau local pour discuter en privé, en groupe et partager des fichiers. Il fonctionne sans serveur ni compte, avec des connexions chiffrées en pair-à-pair et un historique conservé sur votre ordinateur.

Écrit en Rust, avec une interface native egui pour macOS, Windows et Linux.

- Découverte automatique des pairs ; conversations privées et salons.
- Markdown, emojis, réactions, réponses et transferts jusqu'à 2 Gio.
- Notifications système, mode résident et réglage des thèmes, textes et polices.
- La messagerie locale fonctionne hors ligne. Les GIF et le catalogue de polices en ligne, activable sur accord, utilisent Internet.

## Installation

Bêta **1.0.0-beta.2** depuis les [releases GitHub](https://github.com/abnd-corp/Abcom/releases) : paquets pour macOS Apple silicon, Windows x86-64 (ARM64 par émulation) et Linux x86-64. Les autres systèmes passent par Nix ou [les sources](#démarrage-rapide).

> **Non signé, par choix :** ni Developer ID ni notarisation Apple, ni Authenticode, et aucune signature prévue ([ADR 0002](docs/adr/0002-no-code-signing.fr.md)). Vérifiez le fichier avec le `SHA256SUMS.txt` de la release, puis autorisez le premier lancement : sous macOS, Réglages Système → Confidentialité et sécurité → **Ouvrir quand même** ; sous Windows, SmartScreen → **Informations complémentaires** → **Exécuter quand même**. Ne désactivez jamais Gatekeeper ni SmartScreen globalement.

| Système | Commande ou fichier de la release |
| --- | --- |
| macOS · Homebrew | `brew tap abnd-corp/abcom https://github.com/abnd-corp/Abcom` puis `brew install --cask abnd-corp/abcom/abcom` |
| macOS · app | `abcom-macos-aarch64.tar.gz` : déplacer `Abcom.app` dans Applications |
| Windows · WinGet | `winget install --manifest .\winget\Abnd.Abcom.yaml` depuis l’archive `abcom-package-recipes.tar.gz` de la release (manifestes locaux autorisés une fois par un administrateur) |
| Windows · installateur | `abcom-windows-x86_64-setup.exe`, ou l’archive portable `abcom-windows-x86_64.zip` |
| Linux · AppImage | `Abcom-1.0.0-beta.2-x86_64.AppImage` : `chmod +x`, puis le lancer |
| Linux · Flatpak | `flatpak install --user ./Abcom-1.0.0-beta.2-x86_64.flatpak` |
| Nix · Linux, macOS | `nix profile add github:abnd-corp/Abcom/v1.0.0-beta.2#abcom` (compilé depuis les sources) |

Commandes de téléchargement, vérification des empreintes, mises à jour et désinstallation de chaque format : [recettes d’installation](packaging/README.fr.md).

## Démarrage rapide

Installer [Rust](https://rustup.rs/), puis :

```sh
git clone https://github.com/abnd-corp/Abcom.git
cd Abcom
cargo run --release -- alice
```

Lancer Abcom sur un autre ordinateur du même réseau avec un nom différent. Sous Linux, installer les [paquets système](docs/06-installation.md#dépendances-système-sous-linux) nécessaires à la compilation.

Pour essayer deux instances locales sur macOS ou Linux : `make run2`.

## Documentation

[Installation](docs/06-installation.md) · [Guide utilisateur](docs/05-fonctionnalites.md) · [Dépannage et FAQ](docs/11-depannage.md) · [Toute la documentation](docs/README.fr.md)

**Logiciel en bêta.** La vérification d'identité au premier contact et le stockage local non chiffré sont expliqués dans le [guide de sécurité](docs/03-reseau-et-securite.md). Consulter les [limites connues](docs/09-limites-et-pistes.md) et le [changelog](CHANGELOG.md) avant un déploiement.

## Contribuer et licence

Commencer par le [guide de développement](docs/07-developpement.md) et les [règles de contribution](CONTRIBUTING.md). Signaler les problèmes reproductibles dans les [issues GitHub](https://github.com/abnd-corp/Abcom/issues).

Sous licence [GNU AGPL v3](LICENSE). Les ressources embarquées gardent leurs propres licences ; voir les [crédits](docs/07-developpement.md#ressources-embarquées).
