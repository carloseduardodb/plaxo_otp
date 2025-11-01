use std::sync::{Arc, RwLock};

use crate::google_drive::GoogleDriveAuth;
use crate::types::OtpApp;

#[derive(Debug)]
pub struct AppState {
    pub apps: Arc<RwLock<Vec<OtpApp>>>,
    pub master_password: Arc<RwLock<Option<String>>>,
    pub encryption_key: Arc<RwLock<Option<[u8; 32]>>>,
    pub google_auth: Arc<RwLock<Option<GoogleDriveAuth>>>,
    pub syncing: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            apps: Arc::new(RwLock::new(Vec::new())),
            master_password: Arc::new(RwLock::new(None)),
            encryption_key: Arc::new(RwLock::new(None)),
            google_auth: Arc::new(RwLock::new(None)),
            syncing: Arc::new(RwLock::new(false)),
        }
    }

    pub fn get_apps(&self) -> Vec<OtpApp> {
        self.apps.read().unwrap().clone()
    }

    pub fn get_app_by_id(&self, id: &str) -> Option<OtpApp> {
        self.apps.read().unwrap()
            .iter()
            .find(|app| app.id == id)
            .cloned()
    }

    pub fn set_apps(&self, apps: Vec<OtpApp>) {
        let mut apps_guard = self.apps.write().unwrap();
        *apps_guard = apps;
    }

    pub fn add_app(&self, app: OtpApp) {
        let mut apps_guard = self.apps.write().unwrap();
        apps_guard.push(app);
    }

    pub fn remove_app(&self, id: &str) -> bool {
        let mut apps_guard = self.apps.write().unwrap();
        let before_len = apps_guard.len();
        apps_guard.retain(|app| app.id != id);
        apps_guard.len() != before_len
    }

    pub fn update_app_name(&self, id: &str, new_name: String) -> bool {
        let mut apps_guard = self.apps.write().unwrap();
        if let Some(app) = apps_guard.iter_mut().find(|a| a.id == id) {
            app.name = new_name;
            true
        } else {
            false
        }
    }

    pub fn get_encryption_key(&self) -> Option<[u8; 32]> {
        *self.encryption_key.read().unwrap()
    }

    pub fn set_encryption_key(&self, key: [u8; 32]) {
        let mut key_guard = self.encryption_key.write().unwrap();
        *key_guard = Some(key);
    }

    pub fn set_master_password(&self, password: String) {
        let mut password_guard = self.master_password.write().unwrap();
        *password_guard = Some(password);
    }

    pub fn has_master_password(&self) -> bool {
        self.master_password.read().unwrap().is_some()
    }

    pub fn get_google_auth(&self) -> Option<GoogleDriveAuth> {
        self.google_auth.read().unwrap().clone()
    }

    pub fn set_google_auth(&self, auth: Option<GoogleDriveAuth>) {
        let mut auth_guard = self.google_auth.write().unwrap();
        *auth_guard = auth;
    }

    pub fn is_syncing(&self) -> bool {
        *self.syncing.read().unwrap()
    }

    pub fn set_syncing(&self, syncing: bool) {
        let mut syncing_guard = self.syncing.write().unwrap();
        *syncing_guard = syncing;
    }

    pub fn clear_all(&self) {
        let mut apps_guard = self.apps.write().unwrap();
        let mut password_guard = self.master_password.write().unwrap();
        let mut key_guard = self.encryption_key.write().unwrap();
        let mut auth_guard = self.google_auth.write().unwrap();

        apps_guard.clear();
        *password_guard = None;
        *key_guard = None;
        *auth_guard = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
