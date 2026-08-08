#!/bin/bash
# 💎 Siliceo-Nexus — Installer Automatico a 1-Click per Pierino (Linux & macOS)

set -e

echo "======================================================="
echo "  💎 Installazione Automatica di Siliceo-Nexus v0.1.0  "
echo "======================================================="
echo ""

INSTALL_DIR="$HOME/.siliceo-nexus"
BIN_DIR="$INSTALL_DIR/bin"
DATA_DIR="$INSTALL_DIR/data"

mkdir -p "$BIN_DIR" "$DATA_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -f "$SCRIPT_DIR/target/release/siliceo-nexus" ]; then
    echo "📦 Copia dell'eseguibile pre-compilato..."
    cp "$SCRIPT_DIR/target/release/siliceo-nexus" "$BIN_DIR/siliceo-nexus"
elif command -v cargo >/dev/null 2>&1; then
    echo "⚙️ Compilazione eseguibile in corso (attendi qualche secondo)..."
    cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
    cp "$SCRIPT_DIR/target/release/siliceo-nexus" "$BIN_DIR/siliceo-nexus"
else
    echo "❌ Errore: Eseguibile non trovato. Scarica il pacchetto Release con l'eseguibile incluso."
    exit 1
fi

chmod +x "$BIN_DIR/siliceo-nexus"

# Configurazione servizio di sottofondo (Systemd per Linux)
if command -v systemctl >/dev/null 2>&1 && [ -d "$HOME/.config/systemd/user" ]; then
    echo "⚙️ Configurazione servizio di avvio automatico..."
    cat << EOF > "$HOME/.config/systemd/user/siliceo-nexus.service"
[Unit]
Description=Siliceo-Nexus Universal LLM Gateway
After=network.target

[Service]
ExecStart=$BIN_DIR/siliceo-nexus
WorkingDirectory=$INSTALL_DIR
Restart=always
RestartSec=3
Environment=NEXUS_HOST=127.0.0.1:8082

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload
    systemctl --user enable siliceo-nexus.service
    systemctl --user restart siliceo-nexus.service
    echo "🟢 Servizio Siliceo-Nexus attivo in background!"
else
    # Avvio diretto in background
    echo "🚀 Avvio di Siliceo-Nexus in background..."
    nohup "$BIN_DIR/siliceo-nexus" > "$INSTALL_DIR/nexus.log" 2>&1 &
fi

# Creazione Scorciatoia sul Desktop di Pierino
DESKTOP_DIR="$HOME/Desktop"
if [ ! -d "$DESKTOP_DIR" ] && [ -d "$HOME/Scrivania" ]; then
    DESKTOP_DIR="$HOME/Scrivania"
fi

if [ -d "$DESKTOP_DIR" ]; then
    echo "🖥️ Creazione icona sul Desktop..."
    cat << EOF > "$DESKTOP_DIR/Siliceo-Nexus.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=Siliceo-Nexus
Comment=Universal LLM Gateway & Control Dashboard
Exec=xdg-open http://localhost:8082/
Icon=web-browser
Terminal=false
Categories=Utility;Development;
EOF
    chmod +x "$DESKTOP_DIR/Siliceo-Nexus.desktop"
fi

echo ""
echo "======================================================="
echo " 🎉 INSTALLAZIONE COMPLETATA CON SUCCESSO!"
echo " 🌐 Apertura del Pannello di Controllo nel Browser..."
echo "======================================================="
echo ""

sleep 1

if command -v xdg-open >/dev/null 2>&1; then
    xdg-open http://localhost:8082/
elif command -v open >/dev/null 2>&1; then
    open http://localhost:8082/
else
    echo "👉 Apri il tuo browser e visita: http://localhost:8082/"
fi
