# 06 — Installation et déploiement

[English](en/06-installation.md) · **Français** · [Documentation](README.fr.md)

## Prérequis

- Rust via `rustup`, au moins la version déclarée dans `Cargo.toml` : **1.95**.
- Session graphique et pilote compatible. La compilation CI ne remplace pas une validation native par OS.
- Réseau : les machines doivent être sur le même LAN, ports 9000-9001/tcp et 9001/udp ouverts entre elles.

### Dépendances système sous Linux

Elles sont **requises à la compilation**, pas seulement à l'exécution : sans
elles, `cargo build` échoue avant même d'atteindre notre code.

| Besoin | Debian / Ubuntu | Fedora | Arch | Alpine |
|---|---|---|---|---|
| Clavier, fenêtrage | `libxkbcommon-dev` | `libxkbcommon-devel` | `libxkbcommon` | `libxkbcommon-dev` |
| Son des notifications | `libasound2-dev` | `alsa-lib-devel` | `alsa-lib` | `alsa-lib-dev` |

Le tray utilise KSNI/StatusNotifierItem sur D-Bus : ni GTK ni AppIndicator ne sont requis. Il faut un bus de session et un bureau compatible pour afficher l'icône. Les sélecteurs natifs passent par le portail du bureau. La dépendance de son est optionnelle : voir les features ci-dessous.

### Compiler léger : `tray` et `sound`

Les fonctionnalités `tray` et `sound` sont actives par défaut. Désactiver le son retire ALSA ; désactiver le tray retire l'icône résidente. Aucun de ces modes ne demande GTK pour le tray KSNI.

```bash
# Sans icône résidente ni son : ALSA non requis
cargo build --release --no-default-features

# Sans le tray uniquement
cargo build --release --no-default-features --features sound
```

Sans `tray`, fermer la fenêtre quitte réellement l'application au lieu de la
replier. Sans `sound`, le bip de notification disparaît ; les notifications
système, elles, restent (elles passent par D-Bus en Rust pur, sans dépendance C).

### musl et Alpine

**Compilation dynamique contre musl** (Alpine, `x86_64-unknown-linux-musl` sans
lien statique) : fonctionne, avec les paquets du tableau ci-dessus. La configuration doit être validée avec les bibliothèques graphiques et pilotes de la machine cible.

**Binaire entièrement statique** : cette configuration n'est pas prise en charge par la pile graphique actuelle. Le rendu passe par `wgpu`, qui charge
le pilote Vulkan à l'exécution via `dlopen` — or `dlopen` ne fonctionne pas dans
un binaire musl statique. Le son utilise aussi les bibliothèques ALSA de la plateforme. Un binaire statique supposerait d'abandonner l'interface graphique.

En résumé, sur Alpine : installer les paquets, compiler normalement, obtenir un
binaire lié dynamiquement à musl. Réduire la surface avec
`--no-default-features` si le tray et le son ne sont pas souhaités.

## Lancer depuis les sources (toutes plateformes)

```bash
cargo run --release -- <pseudo>
```

Sans argument, le pseudo reprend la variable `USER`/`USERNAME`. Le binaire compilé se trouve dans `target/release/abcom`.

Avec le tray actif, quitter par son menu avant de tester un redémarrage : la croix masque normalement la fenêtre.

Le [workflow de release](../.github/workflows/release.yml) vise Linux x86-64, macOS Apple silicon et Windows x86-64. Vérifier les [releases réellement publiées](https://github.com/abnd-corp/Abcom/releases) pour les téléchargements disponibles. Les binaires macOS nus ne sont pas signés ni notarisés : vérifier leur origine et la somme SHA-256 publiée avant de suivre les autorisations de Confidentialité et sécurité.

### Variables d'environnement

Lues depuis l'environnement ou un fichier `.env` à la racine du répertoire courant :

| Variable | Rôle |
|---|---|
| `ABCOM_PASSPHRASE` | Passphrase de salon : seules les machines partageant la même valeur peuvent se connecter entre elles (handshake `XXpsk3`) |
| `ABCOM_KLIPY_API_KEY` | Clé API Klipy — nécessaire au sélecteur GIF/mèmes/stickers |
| `ABCOM_INSTANCE` | Numéro d'instance pour lancer plusieurs Abcom sur la même machine (ports et données séparés). Plafonné à ce que la plage de ports permet : une valeur trop grande est ramenée au maximum au lieu de déborder sur des ports déjà attribués |

La messagerie locale ne nécessite ni clé Klipy ni passphrase. Ne pas versionner de secret ; les polices en ligne exigent un accord distinct dans les paramètres.

### Tester le P2P en local

```bash
bash scripts/run-multi.sh          # lance plusieurs instances prêtes à se découvrir
# ou manuellement :
ABCOM_INSTANCE=1 cargo run --release -- alice
ABCOM_INSTANCE=2 cargo run --release -- bob
```

Sous PowerShell, définir `$env:ABCOM_INSTANCE = '1'` avant Alice et `'2'` dans le terminal séparé de Bob. Chaque instance garde ses propres réglages.

## Linux

```bash
make install
```

compile en release puis :

- copie le binaire dans `~/.local/bin/abcom` ;
- crée un lanceur desktop dans `~/.local/share/applications/`, pointant sur le chemin absolu du binaire (`%h` est un spécificateur systemd : dans un fichier `.desktop` il n'est pas développé et le raccourci ne lance rien) ;
- supprime un éventuel service systemd installé par une version antérieure.

### Lancement à l'ouverture de session

**Un seul mécanisme, géré par l'application** : une entrée XDG dans
`~/.config/autostart/`, activable et désactivable depuis Paramètres → Général.

Aucun service systemd n'est installé. Les versions antérieures en posaient un
*en plus* de l'entrée XDG : deux instances démarraient et se disputaient les
mêmes ports. Un service systemd serait de toute façon inadapté ici — il
démarrerait l'application hors de toute session graphique.

Pour installer un binaire déjà compilé sur une autre machine : `bash scripts/abcom-install.sh ./target/release/abcom`. Désinstallation : `make uninstall` ou `scripts/uninstall.sh`.

Pare-feu (ufw) :

```bash
sudo ufw allow 9000:9001/tcp
sudo ufw allow 9001/udp
```

Le bureau doit exposer StatusNotifierItem (GNOME peut demander une extension, KDE/XFCE le fournissent généralement). KSNI enregistre l'icône lorsque le watcher apparaît. Si sa création échoue, fermer quitte ; un tray créé en attente du panneau peut ne pas encore être visible. Vérifier le repli et la réouverture sur le bureau cible avant distribution.

## macOS

```bash
cargo run --release -- <pseudo>
```

L'application gère elle-même le mode résident (barre de menus, retrait du Dock quand la fenêtre est cachée) et propose l'autostart via un Launch Agent. Il n'y a pas encore de bundle `.app` : conséquence connue, les notifications système sont attribuées au terminal plutôt qu'à Abcom. Le packaging `.app` (identifiant `com.abend.abcom`, via cargo-bundle ou cargo-packager) est prévu pour la première release.

## Windows

Compiler et installer depuis PowerShell (pas depuis WSL, sinon l'application ne s'affichera pas sur le bureau Windows) :

```powershell
cd C:\chemin\vers\abcom
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows.ps1
```

Le script vérifie `cargo`, compile en release si nécessaire, installe `abcom.exe` dans `%LOCALAPPDATA%\Programs\abcom`, et crée des raccourcis sur le bureau et dans le menu Démarrer. Pour épingler à la barre des tâches : lancer l'application puis clic droit sur son icône → « Épingler ». Un installateur MSI reste à faire.

## Docker (tests multi-pairs)

```bash
cd scripts/docker
docker compose up --build
```

Le [Dockerfile](../scripts/docker/Dockerfile) construit une image Rust avec les dépendances graphiques d'egui ; le compose lance trois services (`alice`, `bob`, `charlie`) en `network_mode: host`. Configuration de test pour un bureau Linux ; nécessite un accès au serveur X11 de l'hôte. Ce n'est ni un serveur de messagerie headless ni une recette portable Docker Desktop.

## Cibles Makefile utiles

| Cible | Effet |
|---|---|
| `make build` / `make release` | Compilation debug / release |
| `make run` | Lance l'application |
| `make run-multi` | Plusieurs instances locales |
| `make install` / `make uninstall` | Installation Linux / retrait |
| `make check` | Formatage, Clippy toutes cibles/features et tests |
| `make test` / `make test-verbose` | Tests (normal / sortie complète) |
| `make test-module M=app::groups` | Tests d'un module |
