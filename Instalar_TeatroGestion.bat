@echo off
chcp 65001 >nul
title Instalador TeatroGestión - Ballet Oficial de Córdoba
color 0E

echo ==============================================================================
echo    TEATROGESTIÓN - BALLET OFICIAL DE CÓRDOBA
echo    Sistema Integral de Personal, Inventario, Repartos, Cues y Stock
echo    Teatro del Libertador Gral. San Martín - Windows 11
echo.
echo    ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
echo ==============================================================================
echo.

set "SCRIPT_DIR=%~dp0"
set "SRC_DIR=%SCRIPT_DIR%TeatroGestion"

if not exist "%SRC_DIR%\TeatroGestion.exe" (
    if exist "%SCRIPT_DIR%TeatroGestion.exe" (
        set "SRC_DIR=%SCRIPT_DIR%"
    ) else if exist "%SRC_DIR%\gestion-app.exe" (
        rem Existe con nombre alternativo
    ) else (
        echo [!] ERROR: No se encontró la carpeta TeatroGestion con los archivos ejecutables.
        echo Asegúrese de ejecutar este instalador directamente desde el pendrive.
        pause
        exit /b 1
    )
)

echo [*] Carpeta de origen: %SRC_DIR%

set "DEST_DIR=C:\TeatroGestion"
mkdir "%DEST_DIR%" 2>nul
if not exist "%DEST_DIR%" (
    set "DEST_DIR=%LOCALAPPDATA%\TeatroGestion"
    mkdir "%DEST_DIR%" 2>nul
)

echo [*] Instalando en: %DEST_DIR%
echo.

if exist "%SRC_DIR%\TeatroGestion.exe" (
    echo   -> Copiando TeatroGestion.exe...
    copy /y "%SRC_DIR%\TeatroGestion.exe" "%DEST_DIR%\TeatroGestion.exe" >nul
) else if exist "%SRC_DIR%\gestion-app.exe" (
    echo   -> Copiando TeatroGestion.exe...
    copy /y "%SRC_DIR%\gestion-app.exe" "%DEST_DIR%\TeatroGestion.exe" >nul
)

if exist "%SRC_DIR%\WebView2Loader.dll" (
    echo   -> Copiando WebView2Loader.dll...
    copy /y "%SRC_DIR%\WebView2Loader.dll" "%DEST_DIR%\WebView2Loader.dll" >nul
)

if exist "%SRC_DIR%\teatrogestion.ico" (
    echo   -> Copiando icono oficial...
    copy /y "%SRC_DIR%\teatrogestion.ico" "%DEST_DIR%\teatrogestion.ico" >nul
)

echo.
echo ==> Creando accesos directos en Windows...

set "ICO_PATH=%DEST_DIR%\teatrogestion.ico"
if not exist "%ICO_PATH%" set "ICO_PATH=%DEST_DIR%\TeatroGestion.exe,0"

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ws = New-Object -ComObject WScript.Shell; ^
   $s = $ws.CreateShortcut([IO.Path]::Combine([Environment]::GetFolderPath('Desktop'), 'TeatroGestión.lnk')); ^
   $s.TargetPath = '%DEST_DIR%\TeatroGestion.exe'; ^
   $s.WorkingDirectory = '%DEST_DIR%'; ^
   $s.IconLocation = '%ICO_PATH%'; ^
   $s.Description = 'TeatroGestión - Ballet Oficial de Córdoba'; ^
   $s.Save(); ^
   $sm = [IO.Path]::Combine([Environment]::GetFolderPath('Programs'), 'TeatroGestión.lnk'); ^
   $s2 = $ws.CreateShortcut($sm); ^
   $s2.TargetPath = '%DEST_DIR%\TeatroGestion.exe'; ^
   $s2.WorkingDirectory = '%DEST_DIR%'; ^
   $s2.IconLocation = '%ICO_PATH%'; ^
   $s2.Description = 'TeatroGestión - Ballet Oficial de Córdoba'; ^
   $s2.Save()"

echo   [OK] Acceso directo en el Escritorio.
echo   [OK] Acceso directo en el Menú Inicio.
echo.
echo ==============================================================================
echo   ¡INSTALACIÓN COMPLETADA EXITOSAMENTE!
echo   TeatroGestión ha quedado instalado en %DEST_DIR%
echo ==============================================================================
echo.
echo   ★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★
echo.

set /p RESP="¿Desea iniciar TeatroGestión ahora? (S/n): "
if /i "%RESP%"=="" set RESP=S
if /i "%RESP%"=="S" (
    start "" "%DEST_DIR%\TeatroGestion.exe"
)

pause
exit /b 0
