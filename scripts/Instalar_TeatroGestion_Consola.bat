@echo off
chcp 65001 >nul
title TeatroGestión — Instalador Consola (Ballet Oficial de Córdoba)

set "SCRIPT_DIR=%~dp0"
set "EXE=%SCRIPT_DIR%Instalar_TeatroGestion.exe"
if not exist "%EXE%" (
    if exist "%SCRIPT_DIR%TeatroGestion\Instalar_TeatroGestion.exe" (
        set "EXE=%SCRIPT_DIR%TeatroGestion\Instalar_TeatroGestion.exe"
    )
)

if exist "%EXE%" (
    "%EXE%" --cli
    exit /b %ERRORLEVEL%
)

echo [!] No se encontró Instalar_TeatroGestion.exe
pause
