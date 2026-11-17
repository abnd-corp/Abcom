#!/bin/bash
# Scénario e2e natif (Linux dans la session de test, ou macOS sur l'hôte) : lance deux instances isolées de
# l'application réelle et vérifie démarrage, découverte, tray et arrêt.
# Usage : scenario_unix.sh BINAIRE DOSSIER_RESULTATS
set -u
BIN=$1
OUT=$2
mkdir -p "$OUT"
export RUST_LOG=abcom=debug
fails=0

check() {
    if eval "$2"; then
        echo "PASS $1"
    else
        echo "FAIL $1"
        fails=$((fails + 1))
    fi
}

# Instances 91 et 92 : dossiers de données et ports distincts de ceux de l'utilisateur.
data_root() {
    if [ "$(uname)" = Darwin ]; then echo "$HOME/Library/Application Support"; else echo "${XDG_DATA_HOME:-$HOME/.local/share}"; fi
}
for n in 91 92; do
    if [ -e "$(data_root)/abcom-$n" ]; then
        echo "FAIL données abcom-$n déjà présentes : scénario refusé pour ne rien écraser"
        exit 2
    fi
done

ABCOM_INSTANCE=91 "$BIN" e2e-a >"$OUT/e2e-a.log" 2>&1 &
a=$!
sleep 3
ABCOM_INSTANCE=92 "$BIN" e2e-b >"$OUT/e2e-b.log" 2>&1 &
b=$!
sleep 25

check "instance A en vie" "kill -0 $a 2>/dev/null"
check "instance B en vie" "kill -0 $b 2>/dev/null"
check "A découvre B" "grep -q 'pair découvert : e2e-b' '$OUT/e2e-a.log'"
check "B découvre A" "grep -q 'pair découvert : e2e-a' '$OUT/e2e-b.log'"
check "aucune panique" "! grep -qiE 'panique|panicked' '$OUT/e2e-a.log' '$OUT/e2e-b.log'"

if command -v busctl >/dev/null 2>&1; then
    busctl --user list >"$OUT/dbus.txt" 2>&1
    if grep -q org.kde.StatusNotifierWatcher "$OUT/dbus.txt"; then
        echo "INFO zone de tray présente"
        check "tray A enregistré" "grep -q 'StatusNotifierItem-$a-' '$OUT/dbus.txt'"
        check "tray B enregistré" "grep -q 'StatusNotifierItem-$b-' '$OUT/dbus.txt'"
    else
        # Sans zone de tray (GNOME standard), la croix doit quitter l'application au lieu de la cacher.
        echo "INFO aucune zone de tray"
        check "sans tray : la croix quittera A" "grep -q 'icône résidente indisponible' '$OUT/e2e-a.log'"
        check "sans tray : la croix quittera B" "grep -q 'icône résidente indisponible' '$OUT/e2e-b.log'"
    fi
else
    echo "INFO tray non vérifié automatiquement sur cette plateforme"
fi

# Capture au mieux : l'outil disponible dépend du bureau.
shot="$OUT/screen.png"
if [ "$(uname)" = Darwin ]; then
    screencapture -x "$shot" 2>/dev/null
elif command -v spectacle >/dev/null 2>&1; then
    spectacle -b -n -f -o "$shot" >/dev/null 2>&1
elif command -v xfce4-screenshooter >/dev/null 2>&1; then
    xfce4-screenshooter -f -s "$shot" >/dev/null 2>&1
elif command -v gnome-screenshot >/dev/null 2>&1; then
    gnome-screenshot -f "$shot" >/dev/null 2>&1
fi
[ -s "$shot" ] && echo "INFO capture : screen.png" || echo "INFO capture indisponible sur ce bureau"

kill -TERM "$a" "$b" 2>/dev/null
for _ in $(seq 1 20); do
    kill -0 "$a" 2>/dev/null || kill -0 "$b" 2>/dev/null || break
    sleep 0.5
done
check "arrêt de A sur SIGTERM" "! kill -0 $a 2>/dev/null"
check "arrêt de B sur SIGTERM" "! kill -0 $b 2>/dev/null"
kill -KILL "$a" "$b" 2>/dev/null

for n in 91 92; do
    cp -R "$(data_root)/abcom-$n/logs" "$OUT/logs-$n" 2>/dev/null
    rm -rf "$(data_root)/abcom-$n"
done

echo "RESULT échecs=$fails"
[ "$fails" -eq 0 ]
