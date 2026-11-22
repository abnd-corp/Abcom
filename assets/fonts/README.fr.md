[English](README.md) · **Français**

# Variantes de polices embarquées

Les polices normales et grasses conservent leurs notices existantes. Les fichiers italiques supplémentaires proviennent des paquets `@fontsource/*` **5.3.0** figés, sous OFL-1.1. [italic-sources.json](italic-sources.json) indique chaque URL, SHA-256, taille et notice applicable. Les 24 fichiers WOFF2 ajoutés représentent **704 512 octets** ; ils sont décodés à la demande puis mis en cache.

| Famille | Italique | Gras italique |
| --- | --- | --- |
| Noto Sans, Inter | Oui | Oui |
| Atkinson Hyperlegible, OpenDyslexic, Literata, Nunito, JetBrains Mono | Oui | Oui |
| Lexend | Inclinaison simulée | Gras incliné |

Les sous-ensembles latin et latin étendu sont inclus lorsqu'ils existent. Les replis pour symboles et glyphes absents restent disponibles. Ces fichiers sont embarqués localement ; choisir une police intégrée ne déclenche aucune requête réseau.

Les notices supplémentaires sont conservées dans [styles/licenses](styles/licenses/). OpenDyslexic utilise sa [notice OFL existante](suggestions/opendyslexic/OFL.txt). Distributions amont : [Fontsource](https://fontsource.org/), [Inter](https://github.com/rsms/inter), [Noto Sans](https://github.com/notofonts/latin-greek-cyrillic).

L'association de variantes personnelles, les replis et les formats acceptés sont décrits dans le [guide utilisateur](../../docs/05-fonctionnalites.md#apparence).
