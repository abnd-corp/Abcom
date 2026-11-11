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
