#!/usr/bin/env bash
# Sincroniza y prepara el pendrive de instalación de TeatroGestión y TeatroCue para Windows y Android.
# Incluye instaladores duales (Gráfico WPF y Consola CLI), manuales oficiales en PDF y Bóveda Documental.
#
# Uso: scripts/sync_usb.sh [PUNTO_DE_MONTAJE]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QLAB_ROOT="/home/droga/Escritorio/qlab"
WIN_RELEASE="${ROOT}/target/x86_64-pc-windows-gnu/release"
QLAB_WIN_RELEASE="${QLAB_ROOT}/target/x86_64-pc-windows-gnu/release"
APK_PATH="${ROOT}/crates/gestion-app/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"
KEYDIR="${HOME}/.config/teatrocue-keys"

die() { echo "ERROR: $*" >&2; exit 1; }
step() { echo; echo "==> $*"; }

USB="${1:-${TEATROGESTION_USB:-}}"
if [ -z "${USB}" ]; then
    mapfile -t CANDIDATES < <(ls -d "/run/media/${USER}"/* "/media/${USER}"/* 2>/dev/null | grep -vE '/Gordo$' || true)
    if [ "${#CANDIDATES[@]}" -eq 0 ] && [ -d "/run/media/${USER}/Gordo" ]; then
        echo "Aviso: No se detectó pendrive extra, usando almacenamiento externo Gordo (/run/media/${USER}/Gordo)."
        CANDIDATES=("/run/media/${USER}/Gordo")
    fi
    case "${#CANDIDATES[@]}" in
        0) die "No se encontró ningún pendrive montado. Inserte el pendrive o especifique la ruta: $0 /run/media/${USER}/PENDRIVE" ;;
        1) USB="${CANDIDATES[0]}" ;;
        *)
            echo "Hay varios dispositivos montados. Elija uno:" >&2
            for c in "${CANDIDATES[@]}"; do echo "  - ${c}" >&2; done
            die "Pase el punto de montaje como argumento: $0 /ruta/al/pendrive"
            ;;
    esac
fi

[ -d "${USB}" ] || die "La ruta ${USB} no existe"
[ -w "${USB}" ] || die "No hay permisos de escritura en ${USB}"

step "Destino del Pendrive: ${USB}"

# 1. TEATROGESTIÓN - Componentes y Binarios
[ -f "${WIN_RELEASE}/Instalar_TeatroGestion.exe" ] || die "Falta Instalar_TeatroGestion.exe."
[ -f "${WIN_RELEASE}/gestion-app.exe" ] || die "Falta gestion-app.exe."
[ -f "${APK_PATH}" ] || die "Falta el APK Android en ${APK_PATH}"

mkdir -p "${USB}/TeatroGestion"

step "Sincronizando TeatroGestión (Instalador Dual, Ejecutable y Scripts)..."
cp -v "${WIN_RELEASE}/Instalar_TeatroGestion.exe" "${USB}/"
cp -v "${WIN_RELEASE}/Instalar_TeatroGestion.exe" "${USB}/TeatroGestion/"
cp -v "${WIN_RELEASE}/gestion-app.exe" "${USB}/TeatroGestion/TeatroGestion.exe"

# Scripts y Lanzadores de TeatroGestión
if [ -f "${ROOT}/scripts/Instalar_TeatroGestion_Grafico.bat" ]; then
    cp -v "${ROOT}/scripts/Instalar_TeatroGestion_Grafico.bat" "${USB}/"
    cp -v "${ROOT}/scripts/Instalar_TeatroGestion_Grafico.bat" "${USB}/TeatroGestion/"
fi
if [ -f "${ROOT}/scripts/Instalar_TeatroGestion_Consola.bat" ]; then
    cp -v "${ROOT}/scripts/Instalar_TeatroGestion_Consola.bat" "${USB}/"
    cp -v "${ROOT}/scripts/Instalar_TeatroGestion_Consola.bat" "${USB}/TeatroGestion/"
fi
if [ -f "${ROOT}/scripts/Instalador_Grafico.ps1" ]; then
    cp -v "${ROOT}/scripts/Instalador_Grafico.ps1" "${USB}/"
    cp -v "${ROOT}/scripts/Instalador_Grafico.ps1" "${USB}/TeatroGestion/"
fi

# Manual de Usuario en PDF de TeatroGestión
if [ -f "${ROOT}/Manual de Usuario TeatroGestion.pdf" ]; then
    cp -v "${ROOT}/Manual de Usuario TeatroGestion.pdf" "${USB}/"
    cp -v "${ROOT}/Manual de Usuario TeatroGestion.pdf" "${USB}/TeatroGestion/"
fi

# Dependencias WebView2 y Recursos
if [ -f "${WIN_RELEASE}/WebView2Loader.dll" ]; then
    cp -v "${WIN_RELEASE}/WebView2Loader.dll" "${USB}/TeatroGestion/"
fi
if [ -f "${ROOT}/teatrogestion.ico" ]; then
    cp -v "${ROOT}/teatrogestion.ico" "${USB}/TeatroGestion/"
fi

# APK Android
cp -v "${APK_PATH}" "${USB}/TeatroGestion/TeatroGestion.apk"

# 2. TEATROCUE - solo los lanzadores. El instalador y los programas de TeatroCue NO se copian desde aquí: el
#    Instalar_TeatroCue.exe del pendrive se arma con `qlab/scripts/build_usb_installer.sh` (clave pública incrustada,
#    sumas de verificación y la carpeta payload). Copiar el de `target/release` lo reemplazaría por uno sin licencia válida.
step "Sincronizando lanzadores de TeatroCue (el instalador y los programas no se tocan)..."
if [ -f "${QLAB_ROOT}/tools/Instalar_TeatroCue_Grafico.bat" ]; then
    cp -v "${QLAB_ROOT}/tools/Instalar_TeatroCue_Grafico.bat" "${USB}/"
fi
if [ -f "${QLAB_ROOT}/tools/Instalar_TeatroCue_Consola.bat" ]; then
    cp -v "${QLAB_ROOT}/tools/Instalar_TeatroCue_Consola.bat" "${USB}/"
fi
if [ -f "${QLAB_ROOT}/tools/Instalador_Grafico_TeatroCue.ps1" ]; then
    cp -v "${QLAB_ROOT}/tools/Instalador_Grafico_TeatroCue.ps1" "${USB}/"
fi

# Llave maestra y base de licencias: NUNCA se pisan. Lo que ya está en el pendrive es lo que vale (la base de licencias real
# es admin/licencias.db y se sincroniza con teatrocue-admin; esta no se copia ni se vacía desde acá).
step "Llave maestra y licencias (solo se agrega lo que falta, nunca se reemplaza)..."
if [ -f "${KEYDIR}/master.key.enc" ]; then
    for dest in "${USB}/master.key.enc" "${USB}/TeatroGestion/master.key.enc"; do
        if [ -e "${dest}" ]; then
            if ! cmp -s "${KEYDIR}/master.key.enc" "${dest}"; then
                echo "AVISO: ${dest} es distinta de la local y NO se reemplazó." >&2
            fi
        else
            cp -nv "${KEYDIR}/master.key.enc" "${dest}"
        fi
    done
fi
if [ -f "${USB}/admin/licencias.db" ]; then
    echo "Licencias: ${USB}/admin/licencias.db (intacta)."
else
    echo "AVISO: el pendrive no tiene admin/licencias.db; no se crea ni se copia ninguna." >&2
fi

# Guía operativa para el operador
cat > "${USB}/LEEME_TEATROGESTION.txt" <<'EOF'
==============================================================================
   TEATROGESTIÓN & TEATROCUE — SUITE TEATRAL DE MISIÓN CRÍTICA
   Ballet Oficial de Córdoba — Teatro del Libertador San Martín
==============================================================================
★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★

OPCIONES DE INSTALACIÓN EN WINDOWS 11 (PC DEL TEATRO / SIN INTERNET):

A. INSTALACIÓN DE TEATROGESTIÓN (Personal, Repartos, Ausentismo y Bóveda):
   1. Opción Gráfica: Doble clic en "Instalar_TeatroGestion_Grafico.bat".
   2. Opción Consola: Doble clic en "Instalar_TeatroGestion_Consola.bat".
   3. Ingrese la contraseña maestra del desarrollador.
   4. Configure la contraseña de la Bóveda de Inspección (Carina Lion) y el tiempo de bloqueo.
   5. Presione Instalar. El acceso directo se crea automáticamente en el Escritorio.

B. RECUPERACIÓN ADMINISTRATIVA DE LA BÓVEDA DE INSPECCIÓN:
   - Si la inspectora olvidó su contraseña:
     1. Abra "Instalar_TeatroGestion_Grafico.bat".
     2. Vaya a la pestaña "Recuperar Contraseña de Bóveda".
     3. Ingrese la clave maestra del desarrollador, defina la nueva clave y confirme.

C. INSTALACIÓN DE TEATROCUE (Sistema de Audio y Cues):
   1. Opción Gráfica: Doble clic en "Instalar_TeatroCue_Grafico.bat".
   2. Opción Consola: Doble clic en "Instalar_TeatroCue_Consola.bat".

D. MANUALES OFICIALES EN PDF (INCLUIDOS EN EL PENDRIVE):
   - "Manual de Usuario TeatroGestion.pdf" (8 páginas completas con guías paso a paso)
   - "Manual de Usuario TeatroCue.pdf"

E. APLICACIÓN ANDROID (TELÉFONOS SAMSUNG GALAXY A53 / TABLETS):
   - Copie "TeatroGestion/TeatroGestion.apk" al dispositivo e instálelo.
   - Conceda permisos de cámara para escaneo de códigos de barra Gaynor Minden.

REACTIVACIÓN DE EMERGENCIA (SI VENCE Y NO TIENE EL PENDRIVE):
1. En la pantalla de bloqueo, copie el código REQ-XXXX-XXXX-XXXX-XXXX-EEC.
2. Envíelo por WhatsApp al desarrollador.
3. Pegue la clave de desbloqueo RESP-... devuelta y presione [Activar].
==============================================================================
EOF

cp -v "${USB}/LEEME_TEATROGESTION.txt" "${USB}/TeatroGestion/LEEME.txt"

step "Sincronizando escrituras con el almacenamiento físico (sync)..."
sync

echo
echo "=============================================================================="
echo "  ¡PENDRIVE SINCRONIZADO EXITOSAMENTE (TEATROGESTIÓN & TEATROCUE)!"
echo "  Ubicación: ${USB}"
echo "=============================================================================="
ls -lh "${USB}/Instalar_TeatroGestion_Grafico.bat" "${USB}/Instalar_TeatroCue_Grafico.bat" "${USB}/Manual de Usuario TeatroGestion.pdf"
