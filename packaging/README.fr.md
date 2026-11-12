[English](README.md) · **Français**

# Installer sans catalogue public de paquets

Aucune soumission à Homebrew/core, microsoft/winget-pkgs ou nixpkgs n’est nécessaire. Les recettes restent avec Abcom. Les recettes binaires exigent de vrais fichiers de release versionnés et leurs SHA-256 ; elles n’annoncent jamais un artefact absent. Consulter les [releases GitHub](https://github.com/abnd-corp/Abcom/releases) pour les téléchargements disponibles ; préparer une recette ne la publie pas.

## GitHub direct et noms dans un catalogue

`winget install --exact --id cjpais.Handy` fonctionne parce que [Handy possède un manifeste dans le catalogue Microsoft](https://github.com/microsoft/winget-pkgs/tree/master/manifests/c/cjpais/Handy). Ce manifeste télécharge l’installateur depuis la release GitHub de Handy. WinGet ne transforme pas un nom de dépôt GitHub arbitraire en application : Abcom exige encore un manifeste local, une source de paquets configurée ou un futur référencement. Une fiche de catalogue peut pointer vers nos fichiers GitHub, sans héberger nos binaires.

Nix peut utiliser directement `github:abnd-corp/Abcom/<révision>#abcom` dès que cette révision contient le flake et est disponible à distance. Homebrew exige un cask dans un tap configuré, ou le script de tap local ci-dessous ; il ne découvre pas automatiquement notre release GitHub. Flatpak exige un bundle ou un dépôt OSTree, pas une URL de dépôt source. Aucun nouveau dépôt GitHub n’est obligatoire, mais `brew install abcom`, `winget install Abnd.Abcom` et `flatpak install corp.abnd.abcom` ne sont pas disponibles partout sans configurer ou référencer une source au préalable.

## Linux AppImage et Flatpak

Les deux formats sont produits localement par `scripts/package_appimage.py` et `scripts/package_flatpak.sh`. Ils ne soumettent rien à AppImageHub ou Flathub. Le workflow Linux dédié crée des artefacts de test ; le workflow de release existant les joint seulement lorsqu’une release est explicitement déclenchée. Avant cela, utiliser les fichiers construits localement.

Après publication effective d’une release, télécharger l’architecture adaptée (`x86_64` ou `aarch64`) depuis GitHub. Par exemple, avec `VERSION` défini sur le numéro publié sans `v` :

```sh
BASE="https://github.com/abnd-corp/Abcom/releases/download/v$VERSION"
ARCH=$(uname -m)
curl --fail --location --remote-name "$BASE/Abcom-$VERSION-$ARCH.AppImage"
curl --fail --location --remote-name "$BASE/SHA256SUMS.txt"
sha256sum --check --ignore-missing SHA256SUMS.txt
chmod +x "Abcom-$VERSION-$ARCH.AppImage"
"./Abcom-$VERSION-$ARCH.AppImage"
```

AppImage ne demande ni catalogue de paquets ni installation de Rust. C’est un exécutable portable, pas un `.deb`/`.rpm` ni une image Docker. Sans FUSE, utiliser `APPIMAGE_EXTRACT_AND_RUN=1 ./Abcom-…AppImage`. Le runtime et les outils de packaging ont des SHA-256 figés dans `linux/appimage-tools.json` ; un artefact amont remplacé fait échouer la vérification. La construction utilise Ubuntu 22.04 comme base de compatibilité. Les bibliothèques de fenêtrage chargées par `dlopen` sont incluses explicitement. Le son utilise PulseAudio ou le service compatible de PipeWire du bureau ; les pilotes GPU restent fournis par l’hôte. Garder le fichier AppImage dans un chemin durable pour le démarrage automatique.

Avec Flatpak déjà installé sous Linux :

```sh
curl --fail --location --remote-name "$BASE/Abcom-$VERSION-$ARCH.flatpak"
# Vérifier avec SHA256SUMS.txt comme ci-dessus.
flatpak install --user "./Abcom-$VERSION-$ARCH.flatpak"
flatpak run corp.abnd.abcom
# Désinstaller sans effacer les données personnelles :
flatpak uninstall --user corp.abnd.abcom
```

Le bundle contient les fichiers de l’application et la référence au runtime Freedesktop 25.08. Flatpak récupère le runtime et les extensions graphiques manquantes depuis Flathub ; Abcom n’y est pas publié. Le fichier seul n’est pas un paquet de runtimes hors ligne et ne fournit pas de mise à jour automatique d’Abcom ; télécharger puis installer explicitement la version suivante. Un dépôt permettant les mises à jour pourra être ajouté sans Flathub, mais un dépôt GitHub source ordinaire n’est pas un dépôt OSTree.

Flatpak autorise le réseau LAN, le graphisme et le son, X11/Wayland et les seuls accès D-Bus nécessaires au tray et aux notifications. Il n’autorise ni tout le dossier personnel ni tout le bus de session. Joindre les fichiers avec le sélecteur utilisant le portail ; un glisser-déposer d’un chemin non accordé peut échouer. Les données restent dans le bac à sable de l’application, séparées des données natives/AppImage. Le démarrage automatique est désactivé et signalé indisponible dans cette version Flatpak, en attendant une intégration au portail Background. Aucun faux lanceur limité au bac à sable n’est créé.

Pour construire sous Linux avec les dépendances de `linux/Dockerfile` :

```sh
cargo build --release --locked
python3 scripts/package_appimage.py target/release/abcom
bash scripts/package_flatpak.sh target/release/abcom
```

Les scripts refusent les architectures incompatibles. Les outils AppImage sont vérifiés par hash et extraits sans FUSE. `test_linux_appimage.sh` et `test_linux_flatpak.sh` utilisent un affichage Xvfb isolé et des données jetables ; le second télécharge son runtime dans une installation utilisateur temporaire. Docker nécessite des capacités de namespaces imbriqués pour l’export et les tests Flatpak, sans dossiers personnels ni socket Docker montés. `stage_linux_source.py` exporte seulement les entrées publiques déclarées, sans `.env`, `.git`, données personnelles ni dossiers de build. Ces essais virtuels ne valident pas tous les bureaux physiques, portails et réseaux LAN Linux.

Exécuter les tests d’installation seulement sur un runner CI ou conteneur jetable ; les deux scripts exigent `CI=true`. L’option `ABCOM_FLATPAK_TEST_INSTALLATION` désigne un cache de runtimes réservé aux essais, jamais une installation Flatpak personnelle. Ces tests Linux n’installent aucune application supplémentaire sur l’hôte macOS.

## Homebrew sur macOS

Pour une release distribuée, extraire `abcom-package-recipes.tar.gz` puis lancer :

```sh
bash homebrew/install-homebrew.sh homebrew/Casks/abcom.rb
```

Le script crée `abnd-corp/abcom-local`, un tap local sans dépôt distant, et installe l’application dans le dossier d’applications configuré dans Homebrew. Le relancer avec la recette d’une nouvelle release réinstalle cette version ; aucune mise à jour de tap distant n’est automatique. Désinstaller avec `brew uninstall --cask abcom`. Les données personnelles sont conservées ; le Launch Agent est retiré. Une préférence de démarrage automatique active réécrit son chemin au prochain lancement.

**Signature :** le bundle possède une signature ad hoc vérifiée, sans certificat Developer ID ni notarisation Apple. Un téléchargement en quarantaine peut donc être bloqué, y compris avec une alerte « endommagé ». Une installation Homebrew réussie ne prouve pas que Gatekeeper accepte l’application. Une release téléchargée exige une autorisation macOS explicite lorsqu’elle est possible, ou une signature Developer ID et une notarisation pour un premier lancement normal. Ne pas désactiver Gatekeeper globalement. Voir [les explications Apple](https://support.apple.com/en-us/102445).

Pour un **build compilé sur son propre Mac**, préparer et tester le cask depuis le dépôt :

```sh
cargo build --release --locked
python3 scripts/package_macos.py target/release/abcom dist/Abcom.app
mkdir -p dist/packages/artifacts
tar -czf dist/packages/artifacts/abcom-macos-aarch64.tar.gz -C dist Abcom.app
python3 scripts/prepare_packages.py --artifacts dist/packages/artifacts
bash scripts/install-homebrew.sh dist/packages/homebrew/Casks/abcom.rb --local-build
open /Applications/Abcom.app
```

`--local-build` accepte seulement une recette locale `file:///`. Après vérification du hash de l’archive et installation par Homebrew, le script vérifie l’identifiant et la signature du bundle, puis retire la quarantaine de téléchargement **uniquement sur cette application**. Aucun réglage global de sécurité n’est modifié. Réserver ce mode au code compilé et jugé fiable ; il ne sert pas à ouvrir une application téléchargée d’origine inconnue. La recette vise actuellement Apple silicon, comme l’artefact macOS de release.

## WinGet sur Windows

La recette vise un installateur Inno Setup par utilisateur, avec raccourci dans le menu Démarrer et désinstallateur. Il conserve les données de l’application. Depuis l’archive de recettes d’une release :

```powershell
winget validate --manifest .\winget\Abnd.Abcom.yaml
winget install --manifest .\winget\Abnd.Abcom.yaml --accept-package-agreements
winget uninstall --id Abnd.Abcom
```

La politique Windows doit autoriser les manifestes locaux. Si elle les désactive, `winget settings --enable LocalManifestFiles` exige un administrateur ; l’installateur lui-même n’en exige pas. Le manifeste référence des fichiers de release HTTPS. Pour un essai non publié, servir ses propres artefacts en HTTP sur localhost puis régénérer avec `--base-url http://127.0.0.1:<port>` ; WinGet ne peut pas utiliser une recette `file://` produite pour un test local Mac. [Documentation des manifestes](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest).

Pour construire sous Windows, installer Inno Setup 6 en plus des outils de compilation Rust/Visual Studio, puis lancer :

```powershell
cargo build --release --locked
./scripts/package_windows.ps1 -BinaryPath target/release/abcom.exe
```

`scripts/test_windows_package.ps1` contrôle installation, réinstallation, raccourci, désinstallation et conservation des données sur un runner Windows jetable. Il ne certifie pas le rendu visuel natif. La CI le lance sous Windows ; aucun test WinGet natif n’est déclaré réalisé depuis macOS.

## Nix sur Linux et macOS

Depuis le dépôt, avec les flakes activés :

```sh
nix build .#abcom
nix profile add .#abcom
# plus tard :
nix profile remove abcom
```

Utiliser un checkout ou commit figé pour une installation distante. `flake.lock` fixe nixpkgs et `Cargo.lock` les crates. Nix fournit la chaîne de compilation, SQLite et, sous Linux, ALSA et les bibliothèques graphiques chargées dynamiquement. Linux reçoit un lanceur et une icône hicolor ; macOS reçoit `Applications/Abcom.app` à l’intérieur du paquet. Ajouter explicitement cette application au bureau si nécessaire ; un profil Nix ne la copie pas automatiquement dans `/Applications`.

La source contient seulement les entrées de compilation listées dans `packaging/nix/package.nix`, jamais `.env`, les données locales ou les dossiers de build. Le bac à sable ne lance pas les tests réseau/fenêtre native ; ceux-ci restent dans `make check` et le cahier de tests. Une session graphique, D-Bus de session, le portail du bureau et un pilote graphique adapté restent nécessaires. Retirer le profil conserve les données personnelles ; désactiver le démarrage automatique avant de désinstaller une installation Nix.

## Maintenance

`scripts/prepare_packages.py --artifacts <dossier> --base-url <url-versionnée>` génère les recettes cask et WinGet avec les hashes des fichiers réels. Il refuse les identifiants dans les URL et le HTTP distant. Le workflow de release joint les recettes, l’installateur Windows et `SHA256SUMS.txt` à la release GitHub ; il ne soumet rien aux catalogues externes.

Le Docker existant à trois pairs reste un outil Linux/X11. Son contexte inclut maintenant le script de build et les exemples. Il ne teste ni WinGet, ni Gatekeeper, ni la barre de menus macOS, ni un vrai panneau Linux. Le bilan des essais locaux figure dans le [journal d’audit](../docs/08-historique-et-audits.md).
