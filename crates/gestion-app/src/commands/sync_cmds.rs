use crate::state::AppState;
use gestion_core::db::sync_repo;
use gestion_core::dto::{SyncBundle, SyncReportDto};
use gestion_core::error::SerializableCoreError;
use gestion_core::services::lan_auth;
use gestion_core::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;

pub const DEFAULT_SYNC_PORT: u16 = 45870;
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static OPEN_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Tope de un paquete de sincronización (pedido o respuesta) y de los encabezados HTTP: un valor declarado enorme ya no
/// reserva memoria ni tumba la aplicación.
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 16 * 1024;
/// Conexiones simultáneas que atiende el servidor (cada una es un hilo).
const MAX_CONNECTIONS: usize = 8;

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn header_value(headers: &str, name: &str) -> Option<String> {
    headers.lines().skip(1).find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim().to_string())
    })
}

fn http_error(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!("{{\"error\":\"{message}\"}}");
    let resp = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    let _ = stream.write_all(resp.as_bytes());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanSyncInfo {
    pub local_ip: String,
    pub port: u16,
    pub is_server_active: bool,
    pub device_name: String,
}

pub fn get_device_name() -> String {
    #[cfg(target_os = "android")]
    {
        "Samsung Galaxy A53 (Depósito / Sala)".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Notebook Inspectora (Windows 11)".to_string()
    }
    #[cfg(not(any(target_os = "android", target_os = "windows")))]
    {
        "Estación de Trabajo (Linux)".to_string()
    }
}

pub fn get_node_id() -> String {
    #[cfg(target_os = "android")]
    {
        "node_a53_mobile".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "node_pc_windows".to_string()
    }
    #[cfg(not(any(target_os = "android", target_os = "windows")))]
    {
        "node_pc_linux".to_string()
    }
}

pub fn detect_local_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "0.0.0.0" {
                    return ip;
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

pub fn start_lan_sync_server(db: Arc<Mutex<DatabaseManager>>, port: u16) {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        let bind_addr = format!("0.0.0.0:{}", port);
        let listener = match TcpListener::bind(&bind_addr) {
            Ok(l) => {
                SERVER_RUNNING.store(true, Ordering::SeqCst);
                println!("[LAN Sync] Servidor P2P activo escuchando en {}", bind_addr);
                l
            }
            Err(e) => {
                eprintln!("[LAN Sync] Error al bindear servidor en {}: {}", bind_addr, e);
                return;
            }
        };
        serve(listener, db);
    });
}

/// Atiende conexiones de `listener` (una por hilo, con tope) hasta que el socket se cierre.
fn serve(listener: TcpListener, db: Arc<Mutex<DatabaseManager>>) {
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if OPEN_CONNECTIONS.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            OPEN_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
            http_error(&mut stream, "503 Service Unavailable", "servidor ocupado");
            continue;
        }
        let db_ref = db.clone();
        thread::spawn(move || {
            let _ = handle_sync_client(&mut stream, db_ref);
            OPEN_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn handle_sync_client(
    stream: &mut TcpStream,
    db: Arc<Mutex<DatabaseManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    let mut header_buf = Vec::new();
    let mut temp = [0u8; 4096];
    let pos = loop {
        let n = stream.read(&mut temp)?;
        if n == 0 {
            return Ok(());
        }
        header_buf.extend_from_slice(&temp[..n]);
        if let Some(pos) = header_buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos;
        }
        if header_buf.len() > MAX_HEADER_BYTES {
            http_error(stream, "431 Request Header Fields Too Large", "encabezados demasiado largos");
            return Ok(());
        }
    };

    let header_str = String::from_utf8_lossy(&header_buf[..pos]).into_owned();
    let first_line = header_str.lines().next().unwrap_or("").trim().to_string();

    // Estado sin datos: no requiere clave y no revela nada del equipo.
    if first_line.starts_with("GET /ping") {
        let body = "{\"status\":\"ok\"}";
        let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if !first_line.starts_with("POST /sync") {
        http_error(stream, "404 Not Found", "no encontrado");
        return Ok(());
    }

    let content_length: usize = header_value(&header_str, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    if content_length > MAX_BODY_BYTES {
        http_error(stream, "413 Payload Too Large", "paquete demasiado grande");
        return Ok(());
    }
    let auth = header_value(&header_str, "authorization").unwrap_or_default();

    // Antes de leer el cuerpo (y mucho antes de tocar la base) se exige la clave de sincronización.
    let key = {
        let guard = db.lock().map_err(|_| "DB lock error")?;
        lan_auth::get_or_create_key(guard.connection()).map_err(|e| format!("clave: {e}"))?
    };

    let mut body = Vec::with_capacity(content_length.min(1 << 20));
    body.extend_from_slice(&header_buf[pos + 4..]);
    while body.len() < content_length {
        let to_read = (content_length - body.len()).min(temp.len());
        let n = stream.read(&mut temp[..to_read])?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&temp[..n]);
    }
    body.truncate(content_length);

    if !lan_auth::verify(&key, &auth, now_secs(), "POST /sync", &body) {
        http_error(stream, "401 Unauthorized", "clave de sincronización incorrecta o pedido vencido");
        return Ok(());
    }

    let remote_bundle: SyncBundle = match serde_json::from_slice(&body) {
        Ok(b) => b,
        Err(_) => {
            http_error(stream, "400 Bad Request", "paquete ilegible");
            return Ok(());
        }
    };
    let device_name = get_device_name();
    let node_id = get_node_id();

    let response_bundle = {
        let mut db_guard = db.lock().map_err(|_| "DB lock error")?;
        // 1. Importar lo recibido (con errores a la vista, no tragados).
        if let Err(e) = sync_repo::import_sync_bundle(db_guard.connection_mut(), &remote_bundle) {
            eprintln!("[LAN Sync] No se pudo importar el paquete recibido: {e}");
        }
        // 2. Devolver lo local actualizado.
        sync_repo::export_sync_bundle(db_guard.connection(), &device_name, &node_id).map_err(|e| format!("Export error: {}", e))?
    };

    let resp_json = serde_json::to_vec(&response_bundle)?;
    let resp_headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-TG-Auth: {}\r\nConnection: close\r\n\r\n",
        resp_json.len(),
        lan_auth::sign(&key, now_secs(), "RESPONSE", &resp_json)
    );
    stream.write_all(resp_headers.as_bytes())?;
    stream.write_all(&resp_json)?;
    Ok(())
}

#[tauri::command]
pub fn get_lan_sync_info() -> Result<LanSyncInfo, SerializableCoreError> {
    Ok(LanSyncInfo {
        local_ip: detect_local_ip(),
        port: DEFAULT_SYNC_PORT,
        is_server_active: SERVER_RUNNING.load(Ordering::SeqCst),
        device_name: get_device_name(),
    })
}

#[tauri::command]
pub fn export_local_sync_bundle(
    state: State<'_, AppState>,
) -> Result<SyncBundle, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error al bloquear la base de datos para exportar".to_string(),
    })?;

    sync_repo::export_sync_bundle(db.connection(), &get_device_name(), &get_node_id())
        .map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn import_remote_sync_bundle(
    state: State<'_, AppState>,
    bundle: SyncBundle,
) -> Result<SyncReportDto, SerializableCoreError> {
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error al bloquear la base de datos para importar".to_string(),
    })?;

    sync_repo::import_sync_bundle(db.connection_mut(), &bundle)
        .map_err(|e| SerializableCoreError::from(&e))
}

/// Envía `post_body` a `target_addr` (`POST /sync`, firmado con `key`) y devuelve el cuerpo de la respuesta, ya verificada.
fn sync_over_tcp(target_addr: &str, key: &str, post_body: &[u8]) -> Result<Vec<u8>, SerializableCoreError> {
    let err = |code: &'static str, message: String| SerializableCoreError { code, message };

    let mut stream = TcpStream::connect_timeout(
        &target_addr.parse().map_err(|e| err("INVALID_IP", format!("Dirección IP o puerto inválido ({}): {}", target_addr, e)))?,
        Duration::from_secs(6),
    )
    .map_err(|e| {
        err(
            "PEER_UNREACHABLE",
            format!("No se pudo conectar con el dispositivo en {}. Verifique que ambos estén en la misma red Wi-Fi y que TeatroGestión esté abierto. ({})", target_addr, e),
        )
    })?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).map_err(|e| err("SOCKET_ERROR", e.to_string()))?;
    stream.set_write_timeout(Some(Duration::from_secs(10))).map_err(|e| err("SOCKET_ERROR", e.to_string()))?;

    let request_header = format!(
        "POST /sync HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAuthorization: {}\r\nConnection: close\r\n\r\n",
        target_addr,
        post_body.len(),
        lan_auth::sign(key, now_secs(), "POST /sync", post_body)
    );
    stream.write_all(request_header.as_bytes()).map_err(|e| err("SEND_ERROR", format!("Error al enviar datos al peer: {}", e)))?;
    stream.write_all(post_body).map_err(|e| err("SEND_ERROR", format!("Error al enviar cuerpo de datos: {}", e)))?;

    // Respuesta: encabezados, luego el cuerpo hasta Content-Length (con tope).
    let mut raw = Vec::new();
    let mut temp = [0u8; 8192];
    let (header_str, body_start) = loop {
        let n = stream.read(&mut temp).map_err(|e| err("RECEIVE_ERROR", format!("Error al leer respuesta del peer: {}", e)))?;
        if n == 0 {
            return Err(err("EMPTY_RESPONSE", "El dispositivo remoto no envió datos de sincronización.".into()));
        }
        raw.extend_from_slice(&temp[..n]);
        if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break (String::from_utf8_lossy(&raw[..pos]).into_owned(), pos + 4);
        }
        if raw.len() > MAX_HEADER_BYTES {
            return Err(err("PARSE_ERROR", "Respuesta inválida del dispositivo remoto.".into()));
        }
    };
    let status: u16 = header_str.lines().next().and_then(|l| l.split_whitespace().nth(1)).and_then(|c| c.parse().ok()).unwrap_or(0);
    if status == 401 {
        return Err(err("SYNC_AUTH", "El otro dispositivo rechazó la clave de sincronización. Escriba en los dos la misma clave (pestaña Sincronización).".into()));
    }
    if status != 200 {
        return Err(err("PEER_ERROR", format!("El dispositivo remoto respondió con el estado {status}.")));
    }
    let content_len: usize = header_value(&header_str, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    if content_len == 0 || content_len > MAX_BODY_BYTES {
        return Err(err("EMPTY_RESPONSE", "El dispositivo remoto no envió datos de sincronización.".into()));
    }
    let mut body = raw[body_start..].to_vec();
    while body.len() < content_len {
        let n = stream.read(&mut temp).map_err(|e| err("RECEIVE_ERROR", format!("Error al leer respuesta del peer: {}", e)))?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&temp[..n]);
    }
    body.truncate(content_len);

    // La respuesta también tiene que venir firmada con la clave: un equipo falso no puede inyectar datos.
    let signed = header_value(&header_str, "x-tg-auth").unwrap_or_default();
    if !lan_auth::verify(key, &signed, now_secs(), "RESPONSE", &body) {
        return Err(err("SYNC_AUTH", "La respuesta del otro dispositivo no está firmada con la clave de sincronización: se descartó.".into()));
    }
    Ok(body)
}

#[tauri::command]
pub fn get_lan_sync_key(state: State<'_, AppState>) -> Result<String, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError { code: "LOCK_ERROR", message: "Error al bloquear la base de datos".to_string() })?;
    lan_auth::get_or_create_key(db.connection()).map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn set_lan_sync_key(state: State<'_, AppState>, key: String) -> Result<String, SerializableCoreError> {
    let db = state.db.lock().map_err(|_| SerializableCoreError { code: "LOCK_ERROR", message: "Error al bloquear la base de datos".to_string() })?;
    lan_auth::set_key(db.connection(), &key).map_err(|e| SerializableCoreError::from(&e))?;
    lan_auth::get_or_create_key(db.connection()).map_err(|e| SerializableCoreError::from(&e))
}

#[tauri::command]
pub fn sync_with_lan_peer(
    state: State<'_, AppState>,
    peer_ip: String,
    peer_port: Option<u16>,
) -> Result<SyncReportDto, SerializableCoreError> {
    let target_port = peer_port.unwrap_or(DEFAULT_SYNC_PORT);
    let target_addr = format!("{}:{}", peer_ip.trim(), target_port);

    // 1. Exportar lo local y tomar la clave
    let (local_bundle, key) = {
        let db = state.db.lock().map_err(|_| SerializableCoreError {
            code: "LOCK_ERROR",
            message: "Fallo al acceder a la base de datos local".to_string(),
        })?;
        let bundle = sync_repo::export_sync_bundle(db.connection(), &get_device_name(), &get_node_id()).map_err(|e| SerializableCoreError::from(&e))?;
        (bundle, lan_auth::get_or_create_key(db.connection()).map_err(|e| SerializableCoreError::from(&e))?)
    };

    let post_body = serde_json::to_vec(&local_bundle).map_err(|e| SerializableCoreError {
        code: "SERIALIZE_ERROR",
        message: format!("Error al serializar paquete de sincronización: {}", e),
    })?;

    // 2. Enviar y recibir (firmado en las dos direcciones)
    let body = sync_over_tcp(&target_addr, &key, &post_body)?;

    let peer_bundle: SyncBundle = serde_json::from_slice(&body).map_err(|e| SerializableCoreError {
        code: "PARSE_ERROR",
        message: format!("Error al deserializar paquete recibido del peer: {}", e),
    })?;

    // 3. Importar lo del otro equipo
    let mut db = state.db.lock().map_err(|_| SerializableCoreError {
        code: "LOCK_ERROR",
        message: "Error al bloquear la base de datos para importar respuesta".to_string(),
    })?;

    sync_repo::import_sync_bundle(db.connection_mut(), &peer_bundle).map_err(|e| SerializableCoreError::from(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_with(db: DatabaseManager) -> (String, Arc<Mutex<DatabaseManager>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let db = Arc::new(Mutex::new(db));
        let d = db.clone();
        thread::spawn(move || serve(listener, d));
        (addr, db)
    }

    fn bundle_json(db: &Arc<Mutex<DatabaseManager>>) -> Vec<u8> {
        let g = db.lock().unwrap();
        serde_json::to_vec(&sync_repo::export_sync_bundle(g.connection(), "prueba", "node_test").unwrap()).unwrap()
    }

    fn raw_request(addr: &str, request: &[u8]) -> String {
        let mut s = TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(request).unwrap();
        let mut out = Vec::new();
        let _ = s.read_to_end(&mut out);
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn two_devices_with_the_same_key_sync_and_everyone_else_on_the_network_is_refused() {
        let (addr, server_db) = server_with(DatabaseManager::open_in_memory().unwrap());
        let key = lan_auth::get_or_create_key(server_db.lock().unwrap().connection()).unwrap();
        let body = bundle_json(&server_db);

        // Con la clave: responde con su paquete, firmado.
        let resp = sync_over_tcp(&addr, &key, &body).expect("sincroniza con la clave correcta");
        assert!(serde_json::from_slice::<SyncBundle>(&resp).is_ok());

        // Con otra clave: 401 y no toca la base.
        let err = sync_over_tcp(&addr, "clave-equivocada-1", &body).unwrap_err();
        assert_eq!(err.code, "SYNC_AUTH", "{}", err.message);

        // Sin encabezado de autenticación: 401 (antes entraba cualquiera y recibía toda la base).
        let raw = format!("POST /sync HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
        let mut req = raw.into_bytes();
        req.extend_from_slice(&body);
        let r = raw_request(&addr, &req);
        assert!(r.starts_with("HTTP/1.1 401"), "{r}");
        assert!(!r.contains("\"records\"") && r.len() < 400, "no devuelve datos: {r}");

        // Pedido firmado hace 10 minutos (repetido por un espía): rechazado.
        let old = lan_auth::sign(&key, now_secs() - 600, "POST /sync", &body);
        let mut req = format!("POST /sync HTTP/1.1\r\nHost: x\r\nAuthorization: {old}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        req.extend_from_slice(&body);
        assert!(raw_request(&addr, &req).starts_with("HTTP/1.1 401"));
    }

    #[test]
    fn a_fake_server_that_answers_without_the_key_is_refused_by_the_client() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let mut buf = [0u8; 65536];
                let _ = s.read(&mut buf);
                let body = b"{\"records\":[]}";
                let _ = s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes());
                let _ = s.write_all(body);
            }
        });
        let err = sync_over_tcp(&addr, "mi-clave-de-sync", b"{}").unwrap_err();
        assert_eq!(err.code, "SYNC_AUTH", "{}", err.message);
    }

    #[test]
    fn oversized_or_malformed_requests_get_an_error_and_cannot_exhaust_memory() {
        let (addr, _db) = server_with(DatabaseManager::open_in_memory().unwrap());
        let r = raw_request(&addr, b"POST /sync HTTP/1.1\r\nContent-Length: 99999999999\r\nConnection: close\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 413"), "{r}");
        let junk = vec![b'a'; MAX_HEADER_BYTES + 5000];
        let r = raw_request(&addr, &junk);
        assert!(r.starts_with("HTTP/1.1 431"), "{r}");
        assert!(raw_request(&addr, b"GET /otra-cosa HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404"));
        // El estado no pide clave ni revela el nombre del equipo.
        let r = raw_request(&addr, b"GET /ping HTTP/1.1\r\n\r\n");
        assert!(r.starts_with("HTTP/1.1 200") && !r.contains("Notebook") && !r.contains("Samsung"), "{r}");
    }
}
