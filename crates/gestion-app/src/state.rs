use gestion_core::DatabaseManager;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<DatabaseManager>>,
}

impl AppState {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }
}
