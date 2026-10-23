@echo off
chcp 65001 >nul
title TeatroGestión — Instalador Gráfico (Windows 11)

set "SCRIPT_DIR=%~dp0"
set "PS_SCRIPT=%SCRIPT_DIR%Instalador_Grafico.ps1"
if not exist "%PS_SCRIPT%" (
    if exist "%SCRIPT_DIR%TeatroGestion\Instalador_Grafico.ps1" (
        set "PS_SCRIPT=%SCRIPT_DIR%TeatroGestion\Instalador_Grafico.ps1"
    )
)

if exist "%PS_SCRIPT%" (
    start "" powershell -NoProfile -ExecutionPolicy Bypass -STA -File "%PS_SCRIPT%"
    exit /b 0
)

if exist "%SCRIPT_DIR%Instalar_TeatroGestion.exe" (
    start "" "%SCRIPT_DIR%Instalar_TeatroGestion.exe" --gui
    exit /b 0
)

if exist "%SCRIPT_DIR%TeatroGestion\Instalar_TeatroGestion.exe" (
    start "" "%SCRIPT_DIR%TeatroGestion\Instalar_TeatroGestion.exe" --gui
    exit /b 0
)

echo [!] No se encontró el instalador gráfico ni el ejecutable.
pause
