[English](README.md) · **Français**

# Adaptateurs D-Bus Linux

Ces sources sont des correctifs locaux de projets amont actifs, pas des versions officielles. Abcom prend en charge leur entretien jusqu'à la publication de versions équivalentes. Ils conservent le tray et l'accessibilité Linux en remplaçant les paquets autonomes non maintenus zvariant, zvariant_derive et zbus_names par zbus 6.

| Source | Version importée | Adaptation locale |
| --- | --- | --- |
| AccessKit : accesskit_unix, accesskit_atspi_common | 0.24.1, 0.21.1 | Types, constructeurs, enregistrement des objets et exécution des connexions zbus 6 |
| AT-SPI : atspi, atspi-common, atspi-proxies | 0.29.0, 0.13.0, 0.13.0 | Codecs et proxies zbus 6 ; décodage explicite et vérifié des événements standard/Qt |
| KSNI | 0.3.7 | Portage amont zbus 6 adapté au manifeste du paquet publié |

Les licences originales sont conservées auprès de chaque paquet. `upstream.json` contient les sommes de contrôle des archives du registre et les références amont. Les fichiers modifiés portent une mention locale. Aucun numéro amont n'est présenté comme une nouvelle version officielle.

Les différences avec chaque archive du registre sont consultables dans `patches/<paquet>.patch`.

Le graphe d'exécution utilise zbus 6. Les tests de protocole AT-SPI utilisent aussi des outils maintenus sur zbus 6 (`zbus-lockstep` 0.8 et `zbus_xml` 6). Abcom n'a besoin d'aucune exception RustSec. Les adaptateurs demandent Rust 1.89 ; le minimum existant d'Abcom reste 1.95.

## Entretien

Suivre [AccessKit](https://github.com/AccessKit/accesskit), [AT-SPI](https://github.com/odilia-app/atspi) et le [portage KSNI](https://github.com/iovxw/ksni/pull/48). Relire leurs changements à chaque mise à jour. Ne pas remplacer un adaptateur local avant que tout son graphe d'exécution soit débarrassé des anciens paquets.

Exécuter `make check`, `make check-dbus` et le test d'intégration D-Bus Linux. `check-dbus` vérifie les trois adaptateurs d'accessibilité, dont les deux runtimes AccessKit séparément, avec des manifestes jetables et les correctifs de l'application. Vérifier tray, menu, badge et arbre AT-SPI sur les bureaux Linux de test. Les correctifs Cargo s'appliquent aussi à la résolution des dépendances macOS/Windows ; ils ne changent que les adaptateurs d'exécution Linux.

Sous Linux, `dbus-run-session -- python3 scripts/test_dbus_adapters.py --ksni` vérifie aussi la bibliothèque KSNI. Son manifeste de test jetable exclut GTK et les portails réservés aux exemples amont.

Dès qu'une version amont intègre le portage, retirer ensemble son entrée `[patch.crates-io]` et ses sources, actualiser le lockfile puis relancer les audits et tests natifs. Supprimer ce dossier lorsque les six correctifs sont devenus inutiles. Voir la [décision](../docs/adr/0001-dbus-maintained.fr.md).
