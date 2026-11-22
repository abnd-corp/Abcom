#!/usr/bin/env bash
# abcom-install.sh — Installation rapide d'Abcom (binaire pré-compilé)
set -euo pipefail

BINARY_SOURCE="${1:-.}"
BINARY_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.local/share/abcom"
APPS_DIR="$HOME/.local/share/applications"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════╗"
echo "║  Installation d'Abcom (v2)         ║"
echo "║  Déploiement rapide - binaire      ║"
echo "╚════════════════════════════════════╝"
echo ""

# ── Cherche le binaire ────────────────────────────────────────────────────────
if [ -f "$BINARY_SOURCE/abcom" ]; then
  BINARY_FILE="$BINARY_SOURCE/abcom"
elif [ -f "$BINARY_SOURCE" ]; then
  BINARY_FILE="$BINARY_SOURCE"
elif [ -f "target/release/abcom" ]; then
  BINARY_FILE="target/release/abcom"
else
  echo "❌ Binaire abcom non trouvé !"
  echo "   Usage: $0 [chemin/vers/abcom]"
  echo "   Exemples:"
  echo "     $0 .                    # Dans le répertoire abcom"
  echo "     $0 ~/Downloads/abcom    # Depuis Downloads"
  exit 1
fi

echo "✓ Binaire trouvé: $BINARY_FILE"
echo ""

# ── Bibliothèques chargées au lancement ───────────────────────────────────────
# winit les ouvre dynamiquement : `ldd` ne les voit pas, et leur absence fait échouer
# le lancement (par exemple libxkbcommon-x11 sur un XFCE minimal).
if command -v ldconfig >/dev/null 2>&1; then
  missing=""
  for lib in libxkbcommon.so.0 libxkbcommon-x11.so.0 libwayland-client.so.0 libwayland-cursor.so.0 \
             libwayland-egl.so.1 libX11.so.6 libXcursor.so.1 libXi.so.6 libXrandr.so.2 libxcb.so.1 \
             libvulkan.so.1 libasound.so.2; do
    ldconfig -p 2>/dev/null | grep -q "$lib" || missing="$missing $lib"
  done
  if [ -n "$missing" ]; then
    echo "⚠️  Bibliothèques système manquantes :$missing"
    echo "   Debian/Ubuntu : sudo apt install libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libwayland-cursor0 libwayland-egl1 libx11-6 libxcursor1 libxi6 libxrandr2 libxcb1 libvulkan1 libasound2t64"
    echo "   Fedora : sudo dnf install libxkbcommon libxkbcommon-x11 libwayland-client libwayland-cursor libwayland-egl libX11 libXcursor libXi libXrandr libxcb vulkan-loader alsa-lib"
    echo "   Arch : sudo pacman -S libxkbcommon libxkbcommon-x11 wayland libx11 libxcursor libxi libxrandr libxcb vulkan-icd-loader alsa-lib"
    echo "   (l'AppImage et le Flatpak embarquent déjà ces bibliothèques)"
    echo ""
  fi
fi

# ── Installation ──────────────────────────────────────────────────────────────
echo "→ Installation..."

# Binaire
mkdir -p "$BINARY_DIR"
cp "$BINARY_FILE" "$BINARY_DIR/abcom"
chmod +x "$BINARY_DIR/abcom"
echo "  ✓ Binaire → $BINARY_DIR/abcom"

# Les archives de release et le dépôt fournissent la même icône de lanceur.
ICON_NAME=chat
for candidate in "$SCRIPT_DIR/../assets/app_icon.png" "$SCRIPT_DIR/assets/app_icon.png" "$(dirname "$BINARY_FILE")/assets/app_icon.png" "$SCRIPT_DIR/abcom-icon.png"; do
  if [ -f "$candidate" ]; then
    mkdir -p "$HOME/.local/share/icons/hicolor/512x512/apps"
    cp "$candidate" "$HOME/.local/share/icons/hicolor/512x512/apps/abcom.png"
    ICON_NAME=abcom
    break
  fi
done

# Dossier de données
mkdir -p "$DATA_DIR"
echo "  ✓ Dossier de données → $DATA_DIR"

# Lancement automatique : aucun service systemd. L'application gère elle-même
# son démarrage à l'ouverture de session (entrée XDG dans ~/.config/autostart),
# réglable depuis Paramètres → Général. Installer les deux mécanismes lançait
# deux instances qui se disputaient les mêmes ports.

# ── Raccourci Menu (Desktop file) ─────────────────────────────────────────────
mkdir -p "$APPS_DIR"
# `%h` est un spécificateur systemd : dans un fichier .desktop il n'est pas
# développé et le raccourci ne lance rien. On écrit le chemin absolu (heredoc
# non quoté), en échappant `$username` qui doit rester pour le shell appelé.
cat > "$APPS_DIR/abcom.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Abcom
Comment=Chat LAN P2P et découverte automatique des pairs
Exec=bash -c 'read -p "Ton pseudonyme: " username; "$BINARY_DIR/abcom" "\$username"'
Icon=$ICON_NAME
Terminal=true
Categories=Network;Chat;
StartupWMClass=abcom
Keywords=chat;lan;p2p;distributed;
EOF
echo "  ✓ Raccourci menu créé"

# ── Ajouter au PATH (si absent) ───────────────────────────────────────────────
if [[ ":$PATH:" != *":$BINARY_DIR:"* ]]; then
  if ! grep -q "export PATH.*$BINARY_DIR" "$HOME/.bashrc" 2>/dev/null; then
    echo "" >> "$HOME/.bashrc"
    echo "# Abcom" >> "$HOME/.bashrc"
    echo "export PATH=\"$HOME/.local/bin:\$PATH\"" >> "$HOME/.bashrc"
    echo "  ✓ PATH mis à jour (redémarre le terminal pour l'appliquer)"
  fi
fi

echo ""
echo "╔════════════════════════════════════╗"
echo "║  ✅ Installation terminée !        ║"
echo "╚════════════════════════════════════╝"
echo ""
echo "🚀 Pour lancer Abcom:"
echo ""
echo "   Méthode 1 (Terminal):"
echo "     $ abcom Alice"
echo ""
echo "   Méthode 2 (Menu/Raccourci):"
echo "     → Applications → Abcom"
echo "     (ou cherche 'Abcom' dans ton app launcher)"
echo ""
echo "📝 Configuration:"
echo "   • Données: $DATA_DIR (base abcom.db, médias, identité)"
echo "   • Démarrage à l'ouverture de session : réglable dans Paramètres → Général"
echo "   • Ports: 9000/tcp (P2P), 9001/udp (découverte)"
echo ""
echo "💡 Prochaines étapes:"
echo "   1. Lance l'app sur chaque machine (avec un pseudo unique)"
echo "   2. La liste des 'Pairs LAN' devrait s'auto-remplir"
echo "   3. Envoie un message → devrait arriver aux autres machines !"
echo ""
