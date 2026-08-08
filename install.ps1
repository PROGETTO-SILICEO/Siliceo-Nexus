# 💎 Siliceo-Nexus — Installer Automatico PowerShell per Pierino (Windows)

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "  💎 Installazione Automatica di Siliceo-Nexus v0.1.0  " -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host ""

$InstallDir = "$env:USERPROFILE\.siliceo-nexus"
$BinDir = "$InstallDir\bin"
$DataDir = "$InstallDir\data"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

if (Test-Path "$ScriptDir\target\release\siliceo-nexus.exe") {
    Write-Host "📦 Copia dell'eseguibile pre-compilato..." -ForegroundColor Green
    Copy-Item "$ScriptDir\target\release\siliceo-nexus.exe" "$BinDir\siliceo-nexus.exe" -Force
} elseif (Test-Path "$ScriptDir\siliceo-nexus.exe") {
    Copy-Item "$ScriptDir\siliceo-nexus.exe" "$BinDir\siliceo-nexus.exe" -Force
} else {
    Write-Host "❌ Errore: Eseguibile siliceo-nexus.exe non trovato nella cartella." -ForegroundColor Red
    Exit
}

# Creazione Scorciatoia sul Desktop di Windows
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = "$DesktopPath\Siliceo-Nexus.lnk"

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = "$BinDir\siliceo-nexus.exe"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Siliceo-Nexus Universal LLM Gateway & Dashboard"
$Shortcut.Save()

# Avvio del servizio in background
Write-Host "🚀 Avvio di Siliceo-Nexus..." -ForegroundColor Green
Start-Process -FilePath "$BinDir\siliceo-nexus.exe" -WorkingDirectory $InstallDir -WindowStyle Hidden

Start-Sleep -Seconds 2

# Apertura automatica nel Browser
Write-Host ""
Write-Host "=======================================================" -ForegroundColor Yellow
Write-Host " 🎉 INSTALLAZIONE COMPLETATA CON SUCCESSO!" -ForegroundColor Yellow
Write-Host " 🌐 Apertura del Pannello di Controllo nel Browser..." -ForegroundColor Yellow
Write-Host "=======================================================" -ForegroundColor Yellow
Write-Host ""

Start-Process "http://localhost:8082/"
