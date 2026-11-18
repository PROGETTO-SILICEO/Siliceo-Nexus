#!/usr/bin/env bash
# Rotazione del log di Nexus: copia, tronca (il servizio tiene il fd in append),
# comprime e conserva 14 giorni. Non tocca il servizio.
set -euo pipefail
LOG="/home/guardiano/Documenti/GitHub/Siliceo-Nexus/nexus.log"
[ -f "$LOG" ] || exit 0
SIZE=$(stat -c%s "$LOG")
[ "$SIZE" -lt 50000000 ] && exit 0
STAMP=$(date +%Y%m%d-%H%M)
cp "$LOG" "$LOG.$STAMP"
truncate -s 0 "$LOG"
gzip -f "$LOG.$STAMP"
chmod 600 "$LOG.$STAMP.gz"
find "$(dirname "$LOG")" -maxdepth 1 -name 'nexus.log.*.gz' -mtime +14 -delete
echo "nexus.log ruotato: $STAMP ($SIZE byte)"
