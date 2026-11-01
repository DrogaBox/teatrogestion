#!/usr/bin/env bash
# Instala o actualiza TeatroGestión para el usuario actual en Linux (CachyOS / Arch / Ubuntu):
# Compila el frontend y el backend Tauri v2, instala el binario en ~/.local/opt/teatrogestion/,
# copia el icono oficial y registra el acceso directo en el menú de aplicaciones y en el Escritorio.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Compilando frontend (ui)..."
if [ -d ui ]; then
    (cd ui && pnpm build)
fi

echo "==> Compilando binario de TeatroGestión para Linux..."
cargo build --release -p gestion-app

OPT="$HOME/.local/opt/teatrogestion"
APPS="$HOME/.local/share/applications"
ICONS="$HOME/.local/share/icons/hicolor/256x256/apps"
DESK="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"

BIN="target/release/gestion-app"
if [ ! -x "$BIN" ] && [ -x "target/release/TeatroGestion" ]; then
    BIN="target/release/TeatroGestion"
fi

[ -x "$BIN" ] || { echo "ERROR: falta el binario compilado en $BIN"; exit 1; }

mkdir -p "$OPT" "$APPS" "$ICONS" "$DESK"

# Instalación segura reemplazando el binario (sin error de "Text file busy" si está abierto)
install -m 755 "$BIN" "$OPT/teatrogestion.nuevo" && mv -f "$OPT/teatrogestion.nuevo" "$OPT/teatrogestion"

if [ -f "crates/gestion-app/icons/128x128@2x.png" ]; then
    install -m 644 "crates/gestion-app/icons/128x128@2x.png" "$ICONS/teatrogestion.png"
elif [ -f "teatrogestion.png" ]; then
    install -m 644 "teatrogestion.png" "$ICONS/teatrogestion.png"
fi

cat > "$APPS/teatrogestion.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Version=1.0
Name=TeatroGestión
Comment=Suite de Gestión Integral para la Inspección del Ballet Oficial de Córdoba
Exec=$OPT/teatrogestion
Icon=teatrogestion
Terminal=false
Categories=Office;Database;
StartupWMClass=teatrogestion
DESKTOP
chmod +x "$APPS/teatrogestion.desktop"

cp "$APPS/teatrogestion.desktop" "$DESK/teatrogestion.desktop"
chmod +x "$DESK/teatrogestion.desktop"
gio set "$DESK/teatrogestion.desktop" metadata::trusted true 2>/dev/null || true

update-desktop-database "$APPS" 2>/dev/null || true
gtk-update-icon-cache -q "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "TeatroGestión $(grep -m1 '^version' Cargo.toml | cut -d\" -f2) instalado o actualizado."
echo "Accesos directos listos en $DESK y en el menú de aplicaciones."
