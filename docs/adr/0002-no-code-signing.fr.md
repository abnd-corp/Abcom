[English](0002-no-code-signing.md) · **Français**

# Pas de signature de code pour les binaires publiés

Statut : accepté, 22 novembre 2026 (issue #91).

Une signature reconnue par les systèmes d'exploitation est un service payant. Un Developer ID macOS et sa notarisation demandent l'Apple Developer Program. Une signature Authenticode Windows demande un certificat commercial, et la réputation SmartScreen favorise les certificats EV, plus chers. Les deux ajoutent un renouvellement et des secrets que la CI de release devrait détenir. Abnd utilise des outils gratuits (charte, R29), et Abcom est une application libre, locale d'abord, distribuée depuis ses releases GitHub.

Les binaires d'Abcom **ne sont pas signés et ne le seront pas**, sur aucune plateforme. Le bundle macOS garde sa signature ad hoc uniquement parce que les Mac Apple silicon refusent d'exécuter du code sans aucune signature. Une signature ad hoc prouve que le bundle est intact, pas qui l'a publié.

L'intégrité repose sur GitHub : chaque release publie `SHA256SUMS.txt` en HTTPS à côté des fichiers, la CI les construit depuis les sources du tag (R41), et Nix reconstruit le même tag depuis les sources. Le README explique comment vérifier un téléchargement et autoriser le premier lancement sous macOS (Confidentialité et sécurité) et sous Windows (SmartScreen), sans désactiver ces protections globalement.

Conséquences :

- Gatekeeper et SmartScreen avertissent au premier lancement. C'est attendu et documenté.
- Les fonctions qui exigent un bundle macOS signé, comme des boutons d'action fiables dans les notifications, restent hors périmètre.
- La distribution passe par le tap Homebrew du projet, un manifeste WinGet joint à chaque release et le flake Nix. Les catalogues officiels qui exigent une application signée ne sont pas visés.

À revoir uniquement par un nouvel ADR, par exemple si un service de signature gratuit et reconnu par les systèmes devient accessible aux projets libres.
