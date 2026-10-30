#!/usr/bin/env bash
# Arma el pendrive de la suite Teatro (TeatroGestión + TeatroCue) para las PC con Windows del teatro.
#
# Compila TeatroGestión (con la migración privada 0014) y TeatroCue (con licencia exigida, vía
# qlab/scripts/build_usb_installer.sh, más admin/teatrocue-admin.exe), comprueba que todos los .exe lleven la misma clave pública que
# ~/.config/teatrocue-keys/public.key, copia al pendrive solo lo que cambió y regenera SHA256SUMS.txt.
# No pide la contraseña maestra: las licencias se emiten en cada PC con los instaladores.
#
# USO: scripts/armar_pendrive_suite.sh [--seco] [--solo-cue] PUNTO_DE_MONTAJE
#   --seco      compila y verifica todo en una carpeta temporal y muestra qué cambiaría; no escribe en el pendrive
#   --solo-cue  no compila TeatroGestión: deja sus archivos del pendrive como están (igual se verifican)
#
# Variables opcionales:
#   QLAB_ROOT          repositorio de TeatroCue (por defecto ../qlab junto a este repositorio)
#   TEATROCUE_KEYDIR   claves del desarrollador (por defecto ~/.config/teatrocue-keys)
#   RESPALDO_DIR       dónde guardar los archivos del pendrive que se reemplazan (por defecto ~/respaldo_pendrive_FECHA)
#   CONSERVAR=1        no borrar la carpeta temporal al terminar
#
# Nunca reemplaza master.key.enc ni licencias.db: si el pendrive tiene otra llave maestra, se detiene.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QLAB_ROOT="${QLAB_ROOT:-$(cd "${ROOT}/.." && pwd)/qlab}"
KEYDIR="${TEATROCUE_KEYDIR:-${HOME}/.config/teatrocue-keys}"
TARGET=x86_64-pc-windows-gnu
TG_RELEASE="${ROOT}/target/${TARGET}/release"
MINGW_BIN="/usr/x86_64-w64-mingw32/bin"
MIGRATION_0014="crates/gestion-core/migrations/0014_official_bop_personnel_clean_roster.sql"

die() { echo; echo "ERROR: $*" >&2; exit 1; }
step() { echo; echo "==> $*"; }

SECO=0
SOLO_CUE=0
USB=""
for a in "$@"; do
    case "${a}" in
        --seco) SECO=1 ;;
        --solo-cue) SOLO_CUE=1 ;;
        -h|--help) sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        -*) die "opción desconocida: ${a}" ;;
        *) [ -z "${USB}" ] || die "sobra el argumento ${a}"; USB="${a}" ;;
    esac
done

for tool in cargo pnpm x86_64-w64-mingw32-gcc x86_64-w64-mingw32-objdump sha256sum grep; do
    command -v "${tool}" >/dev/null || die "falta ${tool}"
done

# ---------------------------------------------------------------------------------------------
# 0. Destino (sin detección automática: el disco externo de trabajo nunca debe confundirse con el pendrive)
# ---------------------------------------------------------------------------------------------
[ -n "${USB}" ] || die "indique el punto de montaje del pendrive: $0 [--seco] /run/media/${USER}/PENDRIVE"
[ -d "${USB}" ] || die "no existe ${USB}"
USB="$(cd "${USB}" && pwd)"
case "${USB}" in
    "/"|"${HOME}"|"${HOME}"/*|"${ROOT}"|"${ROOT}"/*|"${QLAB_ROOT}"|"${QLAB_ROOT}"/*) die "${USB} no parece un pendrive" ;;
esac
[ "${SECO}" = 1 ] || [ -w "${USB}" ] || die "no hay permiso de escritura en ${USB}"
[ -x "${QLAB_ROOT}/scripts/build_usb_installer.sh" ] || die "no se encontró TeatroCue en ${QLAB_ROOT} (defina QLAB_ROOT)"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/armar_pendrive_suite.XXXXXX")"
if [ "${CONSERVAR:-0}" = 1 ]; then
    trap 'echo "carpeta temporal conservada: ${WORK}"' EXIT
else
    trap 'rm -rf "${WORK}"' EXIT
fi
STAGE="${WORK}/pendrive"
mkdir -p "${STAGE}"

echo "Pendrive:      ${USB}$([ "${SECO}" = 1 ] && echo '  (en seco: no se escribe nada)')"
echo "TeatroGestión: ${ROOT} ($(git -C "${ROOT}" rev-parse --short HEAD)$(git -C "${ROOT}" diff --quiet HEAD || echo ', con cambios sin commitear'))"
echo "TeatroCue:     ${QLAB_ROOT} ($(git -C "${QLAB_ROOT}" rev-parse --short HEAD)$(git -C "${QLAB_ROOT}" diff --quiet HEAD || echo ', con cambios sin commitear'))"

# ---------------------------------------------------------------------------------------------
# 1. Claves: un único par, y el del pendrive debe ser el mismo
# ---------------------------------------------------------------------------------------------
step "Claves del desarrollador (${KEYDIR})"
[ -f "${KEYDIR}/public.key" ] && [ -f "${KEYDIR}/master.key.enc" ] \
    || die "faltan ${KEYDIR}/public.key o master.key.enc. No se generan claves desde acá: eso invalidaría las licencias emitidas"
PUB="$(tr -d '[:space:]' < "${KEYDIR}/public.key")"
[[ "${PUB}" =~ ^[0-9a-f]{64}$ ]] || die "${KEYDIR}/public.key no tiene 64 caracteres hexadecimales"
[ "$(head -c 6 "${KEYDIR}/master.key.enc")" = "TCKEY1" ] && [ "$(stat -c %s "${KEYDIR}/master.key.enc")" = 86 ] \
    || die "${KEYDIR}/master.key.enc no tiene el formato esperado (TCKEY1, 86 bytes)"
KEY_SHA="$(sha256sum "${KEYDIR}/master.key.enc" | cut -d' ' -f1)"
REPO_PUB="$(tr -d '[:space:]' < "${ROOT}/crates/gestion-license/public_key.hex")"
[ "${REPO_PUB}" = "${PUB}" ] || die "crates/gestion-license/public_key.hex (${REPO_PUB}) no coincide con public.key (${PUB})"
echo "clave pública:  ${PUB}"
echo "master.key.enc: ${KEY_SHA}"
for k in master.key.enc TeatroGestion/master.key.enc; do
    if [ -f "${USB}/${k}" ]; then
        [ "$(sha256sum "${USB}/${k}" | cut -d' ' -f1)" = "${KEY_SHA}" ] \
            || die "el pendrive tiene otra llave maestra en ${k}: no coincide con ${KEYDIR}/master.key.enc. No se toca nada"
        echo "pendrive ${k}: igual"
    fi
done

# Comprueba que un .exe lleve la clave pública esperada y ninguna otra cadena con forma de clave. No cuentan los
# identificadores de versión de SQLite («AAAA-MM-DD hh:mm:ss <64 hex>») ni las tablas de dígitos.
key_report() {
    local exe="$1" n others
    n="$(LC_ALL=C grep -c -aF "${PUB}" "${exe}" || true)"
    others="$(LC_ALL=C grep -aoE '([0-9]{4}-[0-9]{2}-[0-9]{2} [0-9:]{8} )?[0-9a-f]{64}' "${exe}" \
        | grep -vE '^[0-9]{4}-' | grep '[a-f]' | grep -vxF "${PUB}" | sort -u || true)"
    if [ "${n}" -ge 1 ] && [ -z "${others}" ]; then
        echo "ok"
    elif [ -n "${others}" ]; then
        echo "OTRA CLAVE: $(echo "${others}" | tr '\n' ' ')"
    else
        echo "SIN CLAVE"
    fi
}

step "Claves en los .exe que hoy tiene el pendrive"
while IFS= read -r -d '' exe; do
    printf '  %-45s %s\n' "${exe#"${USB}"/}" "$(key_report "${exe}")"
done < <(find "${USB}" -maxdepth 2 -path "${USB}/.*" -prune -o -type f -iname '*.exe' ! -iname 'ff*.exe' -print0 | sort -z)

# ---------------------------------------------------------------------------------------------
# 2. TeatroGestión para Windows (clave incrustada, migración 0014)
# ---------------------------------------------------------------------------------------------
BUILD_UNIX="$(date +%s)"
if [ "${SOLO_CUE}" = 0 ]; then
    step "TeatroGestión: comprobaciones previas"
    [ -f "${ROOT}/${MIGRATION_0014}" ] || die "falta la migración privada ${MIGRATION_0014} (no está en GitHub; cópiela de la copia de trabajo)"
    grep -q '0014_official_bop_personnel_clean_roster' "${ROOT}/crates/gestion-core/src/db/migrations.rs" \
        || die "migrations.rs no registra la 0014: el plantel no quedaría incluido"
    echo "0014 presente y registrada"

    step "TeatroGestión: interfaz (pnpm)"
    pnpm --dir "${ROOT}/ui" install --frozen-lockfile
    pnpm --dir "${ROOT}/ui" run build

    step "TeatroGestión: compilando para ${TARGET}"
    (
        cd "${ROOT}"
        export TEATROGESTION_PUBLIC_KEY_FILE="${KEYDIR}/public.key"
        export TEATROGESTION_BUILD_UNIX="${BUILD_UNIX}"
        export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
        # CPU genérico y CRT estático, como TeatroCue: las PC de cabina pueden no tener AVX2 ni las DLL de MinGW.
        export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-C target-cpu=x86-64 -C target-feature=+crt-static"
        cargo build --release --target "${TARGET}" -p gestion-app
        TG_PAYLOAD_DIR="${TG_RELEASE}" cargo build --release --target "${TARGET}" -p installer-win -p updater-win
    )
    for f in gestion-app.exe Instalar_TeatroGestion.exe Actualizar_TeatroGestion.exe WebView2Loader.dll; do
        [ -f "${TG_RELEASE}/${f}" ] || die "no se generó ${TG_RELEASE}/${f}"
    done
    # El actualizador incrusta la app: si no es la que se acaba de compilar, actualizaría a otra versión.
    [ "${TG_RELEASE}/Actualizar_TeatroGestion.exe" -nt "${TG_RELEASE}/gestion-app.exe" ] \
        || die "Actualizar_TeatroGestion.exe es anterior a gestion-app.exe: no incrusta la app recién compilada"

    mkdir -p "${STAGE}/TeatroGestion"
    for f in Instalar_TeatroGestion.exe Actualizar_TeatroGestion.exe; do
        cp "${TG_RELEASE}/${f}" "${STAGE}/${f}"
        cp "${TG_RELEASE}/${f}" "${STAGE}/TeatroGestion/${f}"
    done
    cp "${TG_RELEASE}/gestion-app.exe" "${STAGE}/TeatroGestion/TeatroGestion.exe"
    cp "${TG_RELEASE}/WebView2Loader.dll" "${STAGE}/TeatroGestion/WebView2Loader.dll"
    for f in INSTALADOR_SUITE_TEATRO.bat Instalador_Unificado.ps1; do
        [ -f "${ROOT}/scripts/${f}" ] && cp "${ROOT}/scripts/${f}" "${STAGE}/${f}"
    done
else
    step "TeatroGestión: --solo-cue, se conservan los archivos del pendrive"
    for f in Instalar_TeatroGestion.exe Actualizar_TeatroGestion.exe INSTALADOR_SUITE_TEATRO.bat Instalador_Unificado.ps1 \
        TeatroGestion/Instalar_TeatroGestion.exe TeatroGestion/Actualizar_TeatroGestion.exe \
        TeatroGestion/TeatroGestion.exe TeatroGestion/WebView2Loader.dll; do
        if [ -f "${USB}/${f}" ]; then
            mkdir -p "$(dirname "${STAGE}/${f}")"
            cp "${USB}/${f}" "${STAGE}/${f}"
        fi
    done
fi

# ---------------------------------------------------------------------------------------------
# 3. TeatroCue con licencia exigida (su propio script; se arma en una carpeta temporal)
# ---------------------------------------------------------------------------------------------
step "TeatroCue: scripts/build_usb_installer.sh"
mkdir -p "${WORK}/cue"
(cd "${QLAB_ROOT}" && TEATROCUE_KEYDIR="${KEYDIR}" TEATROCUE_BUILD_UNIX="${BUILD_UNIX}" ASSUME_YES=1 \
    scripts/build_usb_installer.sh "${WORK}/cue")
[ "$(sha256sum "${WORK}/cue/master.key.enc" | cut -d' ' -f1)" = "${KEY_SHA}" ] \
    || die "build_usb_installer.sh dejó otra master.key.enc: no se copia nada"
cp "${WORK}/cue/Instalar_TeatroCue.exe" "${STAGE}/"
cp -r "${WORK}/cue/payload" "${STAGE}/payload"

# teatrocue-admin con la clave incrustada: así rechaza una master.key.enc que no corresponda. admin/licencias.db no se toca.
step "TeatroCue: teatrocue-admin.exe"
(cd "${QLAB_ROOT}" && TEATROCUE_PUBLIC_KEY_FILE="${KEYDIR}/public.key" TEATROCUE_BUILD_UNIX="${BUILD_UNIX}" \
    cargo build --release --target "${TARGET}" -p teatrocue-admin)
mkdir -p "${STAGE}/admin"
cp "${QLAB_ROOT}/target/${TARGET}/release/teatrocue-admin.exe" "${STAGE}/admin/teatrocue-admin.exe"

# ---------------------------------------------------------------------------------------------
# 4. Verificación: misma clave en todos los .exe y DLL de MinGW presentes junto a cada uno
# ---------------------------------------------------------------------------------------------
step "Verificando claves en los .exe a copiar"
FAIL=0
while IFS= read -r -d '' exe; do
    rel="${exe#"${STAGE}"/}"
    case "${rel}" in payload/ffmpeg.exe|payload/ffprobe.exe) continue ;; esac
    r="$(key_report "${exe}")"
    printf '  %-45s %s\n' "${rel}" "${r}"
    [ "${r}" = ok ] || FAIL=1
done < <(find "${STAGE}" -type f -iname '*.exe' -print0 | sort -z)
[ "${FAIL}" = 0 ] || die "hay ejecutables sin la clave ${PUB} o con otra: no se copia nada"

step "Verificando DLL que no trae Windows"
while IFS= read -r -d '' exe; do
    dir="$(dirname "${exe}")"
    while read -r dll; do
        lower="${dll,,}"
        if [ "${lower}" = webview2loader.dll ] || [ -f "${MINGW_BIN}/${dll}" ] || [ -f "${MINGW_BIN}/${lower}" ]; then
            [ -n "$(find "${dir}" "${STAGE}/payload" -maxdepth 1 -iname "${dll}" 2>/dev/null | head -1)" ] \
                || die "${exe#"${STAGE}"/} necesita ${dll} y no va en el pendrive"
            echo "  ${exe#"${STAGE}"/}: ${dll} incluida"
        fi
    done < <(x86_64-w64-mingw32-objdump -p "${exe}" | sed -n 's/^\s*DLL Name: //p')
done < <(find "${STAGE}" -type f -iname '*.exe' ! -iname 'ff*.exe' -print0 | sort -z)
echo "  ok"

# master.key.enc solo se agrega donde falta; nunca se reemplaza.
for k in master.key.enc TeatroGestion/master.key.enc; do
    if [ ! -f "${USB}/${k}" ]; then
        mkdir -p "$(dirname "${STAGE}/${k}")"
        cp "${KEYDIR}/master.key.enc" "${STAGE}/${k}"
    fi
done

# ---------------------------------------------------------------------------------------------
# 5. Qué cambia en el pendrive
# ---------------------------------------------------------------------------------------------
step "Cambios en el pendrive"
mapfile -d '' -t STAGED < <(cd "${STAGE}" && find . -type f -printf '%P\0' | sort -z)
CHANGED=()
for rel in "${STAGED[@]}"; do
    new="$(sha256sum "${STAGE}/${rel}" | cut -d' ' -f1)"
    if [ -f "${USB}/${rel}" ]; then
        old="$(sha256sum "${USB}/${rel}" | cut -d' ' -f1)"
        if [ "${old}" = "${new}" ]; then
            printf '  igual   %s\n' "${rel}"
            continue
        fi
        printf '  cambia  %-45s %s -> %s\n' "${rel}" "${old:0:12}" "${new:0:12}"
    else
        printf '  nuevo   %-45s %s\n' "${rel}" "${new:0:12}"
    fi
    CHANGED+=("${rel}")
done
REMOVED=()
if [ -d "${USB}/payload" ]; then
    while IFS= read -r -d '' rel; do
        if [ ! -f "${STAGE}/${rel}" ]; then
            printf '  se quita %s\n' "${rel}"
            REMOVED+=("${rel}")
        fi
    done < <(cd "${USB}" && find payload -type f -print0 | sort -z)
fi

# SHA256SUMS.txt: todo lo que arma este script, la llave maestra y los LEEME de la suite.
SUMS_LIST=("${STAGED[@]}")
for extra in master.key.enc TeatroGestion/master.key.enc LEEME.txt LEEME_SUITE_TEATRO.txt; do
    if [ -f "${USB}/${extra}" ] && [ ! -f "${STAGE}/${extra}" ]; then SUMS_LIST+=("${extra}"); fi
done
mapfile -t SUMS_LIST < <(printf '%s\n' "${SUMS_LIST[@]}" | sort -u)
echo "  SHA256SUMS.txt: ${#SUMS_LIST[@]} entradas"

if [ "${SECO}" = 1 ]; then
    echo
    echo "En seco: ${#CHANGED[@]} archivo(s) cambiarían y ${#REMOVED[@]} se quitarían. No se escribió nada en ${USB}."
    exit 0
fi

if [ "${#CHANGED[@]}" = 0 ] && [ "${#REMOVED[@]}" = 0 ]; then
    echo "  el pendrive ya está al día"
else
    need=0
    for rel in "${CHANGED[@]}"; do need=$(( need + $(stat -c %s "${STAGE}/${rel}") )); done
    avail=$(( $(df --output=avail -B1 "${USB}" | tail -1) ))
    [ "${avail}" -gt "${need}" ] || die "no hay espacio en el pendrive: hacen falta ${need} bytes y quedan ${avail}"

    RESPALDO_DIR="${RESPALDO_DIR:-${HOME}/respaldo_pendrive_$(date +%Y%m%d_%H%M%S)}"
    step "Respaldo de lo que se reemplaza: ${RESPALDO_DIR}"
    for rel in "${CHANGED[@]}" "${REMOVED[@]}" SHA256SUMS.txt admin/SHA256SUMS.txt; do
        if [ -f "${USB}/${rel}" ]; then
            mkdir -p "$(dirname "${RESPALDO_DIR}/${rel}")"
            cp -p "${USB}/${rel}" "${RESPALDO_DIR}/${rel}"
        fi
    done

    step "Copiando al pendrive"
    for rel in "${CHANGED[@]}"; do
        mkdir -p "$(dirname "${USB}/${rel}")"
        cp "${STAGE}/${rel}" "${USB}/${rel}.nuevo"
        mv -f "${USB}/${rel}.nuevo" "${USB}/${rel}"
        echo "  ${rel}"
    done
    for rel in "${REMOVED[@]}"; do
        rm -f "${USB}/${rel}"
        echo "  quitado ${rel}"
    done
fi

(cd "${USB}" && sha256sum "${SUMS_LIST[@]}" > SHA256SUMS.txt.nuevo && mv -f SHA256SUMS.txt.nuevo SHA256SUMS.txt)
(cd "${USB}/admin" && sha256sum teatrocue-admin.exe > SHA256SUMS.txt.nuevo && mv -f SHA256SUMS.txt.nuevo SHA256SUMS.txt)
sync

step "Verificación final en el pendrive"
(cd "${USB}" && sha256sum -c --quiet SHA256SUMS.txt && cd admin && sha256sum -c --quiet SHA256SUMS.txt) \
    || die "SHA256SUMS.txt no coincide con lo copiado"
for rel in "${STAGED[@]}"; do
    cmp -s "${STAGE}/${rel}" "${USB}/${rel}" || die "${rel} en el pendrive no es el que se armó"
done
for k in master.key.enc TeatroGestion/master.key.enc; do
    [ ! -f "${USB}/${k}" ] || [ "$(sha256sum "${USB}/${k}" | cut -d' ' -f1)" = "${KEY_SHA}" ] || die "${k} cambió en el pendrive"
done
while IFS= read -r -d '' exe; do
    r="$(key_report "${exe}")"
    printf '  %-45s %s\n' "${exe#"${USB}"/}" "${r}"
    [ "${r}" = ok ] || die "${exe#"${USB}"/} quedó sin la clave ${PUB}"
done < <(find "${USB}" -maxdepth 2 -path "${USB}/.*" -prune -o -type f -iname '*.exe' ! -iname 'ff*.exe' -print0 | sort -z)

echo
echo "Pendrive listo: ${USB}"
echo "Clave pública en todos los .exe: ${PUB}"
