[English](README.md) · **Français**

# Identité visuelle Abcom

Reconstruction vectorielle de la référence fournie : deux maillons opposés, corail `#FF8A60`, encre `#302830`, crème `#FFF8EF`.

![Aperçu des déclinaisons](preview.png)

| Source SVG dans `svg/` | Usage |
|---|---|
| `abcom-symbol` / `abcom-symbol-on-dark` | Symbole bicolore sur fond clair / sombre |
| `abcom-symbol-ink` / `-cream` / `-coral` | Symbole monochrome |
| `abcom-symbol-outline` | Variante au trait |
| `abcom-wordmark` / `abcom-wordmark-on-dark` | Symbole et lettrage ABCOM converti en tracés |
| `abcom-app-light` / `abcom-app-dark` | Icônes d’application arrondies |
| `abcom-dock` | Icône d’application claire pour le Dock macOS |
| `abcom-tray-light` / `abcom-tray-dark` | Symbole de tray encre / crème |
| `abcom-tray-template` | Masque alpha noir pour une image modèle macOS |

Les PNG sont dans `png/`, sur fond transparent. Icônes d’application : 16 à 1024 px. Tray : 16, 24, 32, 48 et 64 px ; utiliser 32, 48 et 64 px pour les versions 2× de 16, 24 et 32 pixels logiques. Logotypes : 540, 1080 et 2160 px de large. Le lettrage SVG est converti en tracés et ne dépend d’aucune police installée.

Conserver les proportions et l’espace entre les maillons. L’application utilise l’icône couleur claire pour sa fenêtre et son lanceur. macOS utilise la variante Dock haute définition et un masque alpha pour la barre de menus, badge non lu compris ; AppKit l’adapte aux fonds clairs et sombres. Windows et Linux utilisent une icône de tray couleur avec une pastille corail pour les non-lus.

Les fichiers natifs sont `../app_icon.png` (512 px), `../app_icon.ico` (16 à 256 px) et `../app_icon.icns` (représentations standard/Retina). La dépendance de compilation `winresource` intègre l’ICO dans les exécutables Windows ; les raccourcis l’utilisent aussi. `scripts/package_macos.py` crée le bundle `.app` avec ICNS et métadonnées. Les installateurs Linux copient le PNG dans le thème hicolor de l’utilisateur.

Pour régénérer depuis la racine du dépôt, avec la dépendance SVG optionnelle existante :

```sh
python3 scripts/generate_branding.py
cargo run --locked --example export_branding --features avatar-svg
python3 scripts/build_native_icons.py
```

`exports.json` donne les noms et dimensions exacts ; `palette.json` contient la palette. Le moteur vérifie que les exports sont visibles, que le fond des icônes reste transparent et que le masque reste monochrome. Contrôler `preview.png` et les PNG du tray à leur taille réelle après toute modification des tracés.

La régénération ICNS utilise `iconutil` sous macOS ; les autres systèmes conservent le fichier ICNS versionné. Voir les recommandations Apple sur les [iconsets](https://developer.apple.com/library/archive/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html) et les [images modèles](https://developer.apple.com/documentation/appkit/nsimage/istemplate).

Sous la [licence du dépôt](../../LICENSE). Le lettrage utilise la [police Inter embarquée](../fonts/Inter-OFL.txt).
