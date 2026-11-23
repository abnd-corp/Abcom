# 12 — Mesurer les performances

[English](en/12-performance.md) · **Français** · [Documentation](README.fr.md)

## Préparer une mesure reproductible

Compiler avec `cargo build --release --locked`. Noter le commit, la version de Rust, les fonctionnalités, l'OS, le matériel, le pilote, la densité d'écran et la taille de fenêtre. Utiliser deux instances et des données jetables, identiques avant et après le changement.

Mesurer au moins trois fois chaque scénario ; publier les valeurs brutes et la dispersion. Séparer démarrage à froid, lancement à chaud et régime établi. Une cible de performance est un objectif à convenir, pas un résultat acquis.

## Scénarios

| Scénario | Observations |
|---|---|
| Démarrage, puis 100 messages | RSS et temps de démarrage ; préciser si les polices et médias étaient déjà en cache |
| Repos pendant 60 secondes | CPU moyen, GPU et réveils ; refaire au premier plan, en arrière-plan puis fenêtre masquée |
| Trois pages de recherche GIF, puis 10 minutes de repos | RSS, textures relâchées et activité résiduelle ; préciser le réseau et les contenus |
| Défilement de 500 messages avec Markdown et médias | Temps de frame médian et p95, fluidité et coût du parsing/caches |
| Réception et rafale de messages | Latence visible, coût des événements, écritures du thread de stockage, absence de blocage UI |
| Transfert d'un fichier de 1 Go | Débit, CPU/RSS des deux pairs, disque et réseau utilisés |
| Repli puis restauration | Rendu au repos, réception fenêtre cachée, délai de réouverture |
| Usage prolongé | Taille de la base et des médias, croissance des caches, threads et ressources |

## Outils et interprétation

Sur macOS, `ps -o rss= -p <pid>` mesure la RSS et Instruments aide à profiler allocations et CPU. Le Moniteur d'activité donne des observations CPU/GPU. Sur les autres OS, employer un outil équivalent et conserver sa version et ses unités. Ne pas comparer des métriques différentes comme RSS et mémoire allouée.

Distinguer ce qui a été mesuré de ce qui est supposé : un pic n'est pas une moyenne, une machine n'est pas une validation multi-OS. L'historique des chiffres est dans le [journal de maintenance](08-historique-et-audits.md).

Les anciennes recommandations de quota automatique ne s'appliquent plus : les pièces jointes ne sont purgées qu'à la demande. Mesurer leur croissance sans détruire des données d'usage. La mesure ne doit pas changer la politique de conservation pour obtenir un meilleur chiffre.

## Vérification Wayland

Wayland est utilisé nativement lorsqu’il est disponible ; X11 reste pris en charge par la sélection automatique de winit. Le code ne définit plus `WINIT_UNIX_BACKEND`, variable retirée de winit. Les correctifs de boucle CPU (eframe 0.36.2, #8398) et de callbacks bloqués (0.37.0, #8631) sont inclus dans la version utilisée ; aucun changement de dépendance n’est requis.

Le workflow Linux lance `scripts/test_linux_wayland.py` sous Weston headless, sans `DISPLAY`, avec des données jetables et le rendu logiciel Vulkan. Il vérifie qu’une surface Wayland reçoit un buffer, mesure le CPU au repos, suspend le compositeur pour bloquer les callbacks, puis le reprend. Chaque mesure dure 12 secondes après échauffement ; le seuil de régression est 20 % d’un cœur. Ce seuil détecte une boucle active, pas une promesse de consommation sur tout matériel. Suspendre le compositeur ne remplace pas un essai de fenêtre minimisée dans chaque bureau Linux. Les essais X11 sous Xvfb restent séparés.

Dans un conteneur Linux jetable doté de Weston, D-Bus et Vulkan logiciel :

```sh
CI=true dbus-run-session -- python3 scripts/test_linux_wayland.py target/release/abcom
```
