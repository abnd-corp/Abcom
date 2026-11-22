[English](0001-dbus-maintained.md) · **Français**

# Entretien de D-Bus Linux

Statut : proposition sur la branche de l'issue #59, le 16 novembre 2026.

AccessKit et KSNI sont actifs, mais leurs versions publiées utilisent encore zbus 5. RustSec déclare non maintenus les paquets autonomes zvariant, zvariant_derive et zbus_names depuis leur intégration à zbus 6. Une simple mise à jour ne migre pas les API des consommateurs. Remplacer uniquement le tray laisserait la chaîne d'accessibilité inchangée.

Conserver les intégrations existantes et maintenir six correctifs de sources limités aux API zbus 6. Garder les licences et la provenance du registre dans [vendor](../../vendor/README.fr.md). Le portage KSNI reprend sa PR amont ; AccessKit et AT-SPI sont adaptés localement. Abcom assume donc cet entretien ; ces correctifs ne bénéficient pas d'un support amont établi.

Conserver la désérialisation vérifiée des formats d'événements AT-SPI standard et Qt, l'arbre d'accessibilité et StatusNotifier sous Wayland et X11. Ne pas désactiver l'accessibilité, réintroduire GTK3/AppIndicator ou choisir un tray limité à X11 pour faire disparaître un avis d'audit.

Ne retirer les trois exceptions qu'avec toute l'ancienne chaîne d'exécution. Vérifier formatage, Clippy, tests, audits, présence des sources dans les paquets, accessibilité et tray Linux, et fonctionnement macOS/Windows. Les VM couvrent des écrans virtuels ; GPU physiques et transitions entre moniteurs restent des contrôles distincts.

Réexaminer les correctifs à chaque mise à jour amont et les supprimer lorsque les versions compatibles intègrent les changements équivalents. Garder #59 ouverte jusqu'à l'acceptation de l'implémentation et des vérifications dans dev. La compilation d'un prototype ne suffit pas à la fermer.
