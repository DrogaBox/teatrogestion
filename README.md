# TeatroGestión

Sistema de gestión integral para la Inspección del Ballet Oficial de Córdoba (Teatro del Libertador San Martín). Desarrollado en Rust con Tauri v2, SQLite local en WAL mode, sincronización P2P en red local y cliente móvil para Android. Funciona 100% offline, sin servidores externos ni conexión a internet.

---

## Qué hace

- **Cronograma y Citaciones diarias**:
  - Armado de partes de ensayo por bloques horarios dinámicos (10:30 a 13:45 h jornada ordinaria, extensión hasta las 14:45 h en régimen pre-función).
  - Asignación por salas (Sala 1, Sala 3 y Escenario) dividida por sexos, solistas, parejas y refuerzos.
  - Columna operativa **SONIDO** para coordinación directa con la cabina de TeatroCue.
  - Motor anti-cruces en tiempo real ($O(N \log N)$): audita que ningún bailarín, maestro o pianista quede asignado a dos salas o actividades simultáneas, vigila el tope de jornada y valida el descanso mínimo reglamentario inter-jornada (12 h).
  - Impresión de reportes y partes oficiales con la cláusula legal:  
    *«REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.»*

- **Plantel Oficial y Asistencias**:
  - Base de datos oficial cargada con los 64 integrantes del Ballet Oficial de Córdoba (Solistas, Refuerzos, Cuerpo de Baile, Maestros, Pianistas y personal pasivo en régimen jubilatorio).
  - Marcada diaria de fichaje (08:45 h), clase técnica matutina (09:00 a 10:15 h simultánea) y control de rotación de docentes y pianistas.
  - Registro de licencias médicas, partes de kinesiología y francos compensatorios.

- **TeatroStock (Calzado e Indumentaria Técnica)**:
  - Zapatillas de media punta de varón y mujer (Sansha, Capezio, Bloch, HD; cuero vs. tela elástica, estados teñidos, numeración americana y europea).
  - Puntas de danza clásica:
    - **Parser de Gaynor Minden**: descompone códigos alfanuméricos de fábrica (ej. `SC - 7.5W4PDH`, `SK - 8.5M4HDL`) extrayendo línea de manufactura (Europeas ▲, Lyra ●, Americanas ★), modelo, talle, ancho, caja, dureza de vástago/bolsa y talón, con soporte para capelladas altas (+6 mm / +1/2").
    - Bloch (modelos BMS III y ETU) y Sansha Duval.
  - Indumentaria y accesorios: suspensores, medias mallas, cancán y bombachas de escena.
  - Circuito de valeras nominales con control de cupo anual por categoría de revista y descuento atómico del stock físico en depósito.

- **Bóveda de Inspección**:
  - Área segura protegida por contraseña para resguardo de planillas de inspección, observaciones y datos sensibles.
  - Bloqueo configurable por inactividad.
  - Recuperación administrativa sin pérdida de datos validada con la llave maestra `master.key.enc` del pendrive oficial.

- **Sincronización P2P Offline y Respaldo**:
  - Descubrimiento local por mDNS / UDP broadcast en la red Wi-Fi del teatro: sincroniza el celular del depósito o la sala con la PC de la oficina sin tocar internet.
  - Exportación e importación de copias de seguridad en un archivo único empaquetado (`.teatrodata`).

---

## Estructura del workspace

| Crate / Carpeta | Función |
|---|---|
| `crates/gestion-core` | Biblioteca Rust pura (headless). Persistencia SQLite (`rusqlite` con WAL mode y foreign keys activas), migraciones deterministas verificadas por SHA-256, structs de dominio, parser Gaynor Minden y motor anti-cruces. |
| `crates/gestion-app` | Shell multiplataforma en Tauri v2. Comandos IPC para la interfaz de escritorio (Windows x86_64) y móvil (Android ARM64 / Samsung Galaxy A15). |
| `crates/gestion-license` | Licenciamiento offline por hardware (Machine ID) firmado con Ed25519, control de marcas de reloj contra adulteraciones de fecha y activación de emergencia. |
| `crates/installer-win` | `Instalar_TeatroGestion.exe`: instalador nativo para Windows 10/11, configuración de accesos directos, emisión de licencias y setup de bóveda. |
| `crates/updater-win` | `Actualizar_TeatroGestion.exe`: actualizador atómico que aplica ejecutables nuevos y ejecuta las migraciones pendientes sin modificar la base histórica. |
| `ui/` | Frontend web reactivo en Svelte + TypeScript + Vite, adaptado para monitor de escritorio y pantallas táctiles móviles. |
| `scripts/` | Herramientas de despliegue, generadores de manuales en PDF y scripts del instalador unificado. |

---

## Compilar y probar

Requiere Rust estable (`rustup`) y Node.js para el frontend.

```bash
# Correr la suite de pruebas del motor de dominio y base de datos
cargo test -p gestion-core

# Verificar compilación de la app
cargo check --workspace

# Compilar instalador y actualizador para Windows (desde Linux o macOS con zig)
cargo zigbuild --target x86_64-pc-windows-gnu --release -p installer-win -p updater-win
```

---

## Instalador Unificado y Pendrive de la Suite

TeatroGestión comparte el medio de distribución USB con **TeatroCue** para instalar y actualizar ambos sistemas en las máquinas del teatro de forma unificada:

- `INSTALADOR_SUITE_TEATRO.bat`: lanzador en Windows con elevación automática de permisos (UAC) y fallback a consola CMD.
- `Instalador_Unificado.ps1`: interfaz gráfica en Windows Forms (compatible con Windows 7, 10 y 11) que detecta instalaciones existentes y permite actualizar en 1 clic manteniendo intactas todas las bases de datos de personal y sonido.
- `master.key.enc`: llave maestra cifrada para emitir licencias offline y resetear la bóveda de inspección en caso de olvido.
- `licencias.db`: registro SQLite centralizado de equipos autorizados.
- `TeatroGestion/TeatroGestion.apk`: instalador para dispositivos móviles Android (Samsung Galaxy A15 / A53).

---

## Seguridad y Licencias

El sistema utiliza firmas criptográficas Ed25519 asociadas a la placa madre y almacenamiento del equipo (Machine ID). La protección offline incluye marcas monotónicas en SQLite y en el registro para evitar adulteraciones de la fecha del sistema operativo.

---

## Verificación de integridad de la base de datos

1. Cerrar la aplicación y copiar la base a un lugar seguro junto a sus archivos adjuntos si existen (`teatro_gestion.sqlite` y, si aparecen, `teatro_gestion.sqlite-wal` / `-shm`); en Windows vive en `%APPDATA%\com.teatrogestion.app\` y en Linux en `~/.local/share/com.teatrogestion.app/`.
2. Con `sqlite3` instalado (en Windows, de [sqlite.org/download](https://www.sqlite.org/download.html), binarios precompilados), correr desde la carpeta del proyecto: `sqlite3 -readonly "COPIA_teatro_gestion.sqlite" ".read scripts/db_integrity_check.sql"`.
3. Resultado esperado: `integrity_check` = `ok`; `foreign_key_check` sin filas; todas las filas huérfanas en `0`; la lista de migraciones termina en `15 | 0015_leave_type`; y el padrón muestra el total de personas activas e inactivas.
4. El script es de solo lectura, pero correrlo siempre sobre la copia, nunca sobre el archivo original en uso.
