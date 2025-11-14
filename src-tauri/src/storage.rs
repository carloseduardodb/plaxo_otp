use std::fs;
use std::path::PathBuf;

use crate::crypto::{encrypt_data, decrypt_data};
use crate::types::{OtpApp, AppError, Result};

const DATA_DIR: &str = ".plaxo-otp";
const APPS_FILE: &str = "apps.enc";
const GOOGLE_AUTH_FILE: &str = "google_auth.enc";

pub struct Storage;

impl Storage {
    pub fn new() -> Self {
        Self
    }

    fn get_data_dir() -> Result<PathBuf> {
        let mut path = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| AppError::Io("Could not determine home directory".to_string()))?;
        
        path.push(DATA_DIR);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn get_apps_file_path() -> Result<PathBuf> {
        let mut path = Self::get_data_dir()?;
        path.push(APPS_FILE);
        Ok(path)
    }

    fn get_google_auth_file_path() -> Result<PathBuf> {
        let mut path = Self::get_data_dir()?;
        path.push(GOOGLE_AUTH_FILE);
        Ok(path)
    }

    pub fn save_apps(&self, apps: &[OtpApp], key: &[u8; 32]) -> Result<()> {
        tracing::info!("Starting save of {} apps", apps.len());
        
        let json = serde_json::to_string(apps)
            .map_err(|e| {
                tracing::error!("Failed to serialize apps: {}", e);
                e
            })?;
        
        tracing::debug!("Serialized {} bytes of JSON", json.len());
        
        let encrypted = encrypt_data(&json, key)
            .map_err(|e| {
                tracing::error!("Failed to encrypt data: {}", e);
                e
            })?;
        
        tracing::debug!("Encrypted {} bytes", encrypted.len());
        
        let file_path = Self::get_apps_file_path()?;
        tracing::debug!("Target file: {:?}", file_path);
        
        // Secure write with temporary file
        let temp_path = format!("{}.tmp", file_path.to_string_lossy());
        
        // Backup current file if it exists
        if file_path.exists() {
            let backup_path = format!("{}.backup", file_path.to_string_lossy());
            fs::copy(&file_path, &backup_path)
                .map_err(|e| {
                    tracing::warn!("Failed to create backup: {}", e);
                    e
                })?;
            tracing::debug!("Backup created");
        }
        
        // Write to temporary file
        fs::write(&temp_path, &encrypted)
            .map_err(|e| {
                tracing::error!("Failed to write temp file: {}", e);
                e
            })?;
        
        tracing::debug!("Temp file written");
        
        // Atomic move to final file
        fs::rename(&temp_path, &file_path)
            .map_err(|e| {
                tracing::error!("Failed to rename temp file: {}", e);
                e
            })?;
        
        tracing::info!("Successfully saved {} apps to {:?}", apps.len(), file_path);
        Ok(())
    }

    pub fn load_apps(&self, key: &[u8; 32]) -> Result<Vec<OtpApp>> {
        let file_path = Self::get_apps_file_path()?;
        
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        // Try to load main file
        match self.try_load_file(&file_path, key) {
            Ok(apps) => {
                tracing::info!("Loaded {} apps from storage", apps.len());
                Ok(apps)
            }
            Err(e) => {
                tracing::warn!("Failed to load main file: {}", e);
                
                // Try to load from backup
                let backup_path = format!("{}.backup", file_path.to_string_lossy());
                if std::path::Path::new(&backup_path).exists() {
                    tracing::info!("Attempting to load from backup...");
                    match self.try_load_file(&PathBuf::from(&backup_path), key) {
                        Ok(apps) => {
                            tracing::info!("Backup loaded successfully, restoring main file...");
                            // Restore main file from backup
                            if let Err(restore_err) = fs::copy(&backup_path, &file_path) {
                                tracing::warn!("Could not restore main file: {}", restore_err);
                            }
                            Ok(apps)
                        }
                        Err(backup_err) => {
                            tracing::error!("Backup also corrupted: {}", backup_err);
                            Err(e) // Return original error
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    fn try_load_file(&self, file_path: &PathBuf, key: &[u8; 32]) -> Result<Vec<OtpApp>> {
        let encrypted_data = fs::read_to_string(file_path)?;
        let decrypted = decrypt_data(&encrypted_data, key)?;
        let apps: Vec<OtpApp> = serde_json::from_str(&decrypted)?;
        Ok(apps)
    }

    pub fn save_google_auth(&self, auth_data: &str, key: &[u8; 32]) -> Result<()> {
        let encrypted = encrypt_data(auth_data, key)?;
        let file_path = Self::get_google_auth_file_path()?;
        fs::write(&file_path, &encrypted)?;
        tracing::info!("Saved Google auth to storage");
        Ok(())
    }

    pub fn load_google_auth(&self, key: &[u8; 32]) -> Result<String> {
        let file_path = Self::get_google_auth_file_path()?;
        
        if !file_path.exists() {
            return Err(AppError::GoogleDrive("Auth not found".to_string()));
        }
        
        let encrypted_data = fs::read_to_string(&file_path)?;
        let decrypted = decrypt_data(&encrypted_data, key)?;
        tracing::info!("Loaded Google auth from storage");
        Ok(decrypted)
    }

    pub fn clear_google_auth(&self) -> Result<()> {
        let file_path = Self::get_google_auth_file_path()?;
        if file_path.exists() {
            fs::remove_file(&file_path)?;
            tracing::info!("Cleared Google auth from storage");
        }
        Ok(())
    }

    pub fn has_apps_file(&self) -> bool {
        Self::get_apps_file_path()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    pub fn reset_all_data(&self) -> Result<()> {
        let data_dir = Self::get_data_dir()?;
        
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir)?;
            tracing::info!("Reset all data");
        }
        
        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::derive_key;
    use tempfile::TempDir;

    #[test]
    fn test_save_load_apps() {
        let _temp_dir = TempDir::new().unwrap();
        let storage = Storage::new();
        let key = derive_key("test_password");
        
        let apps = vec![
            OtpApp {
                id: "1".to_string(),
                name: "Test App".to_string(),
                secret: "JBSWY3DPEHPK3PXP".to_string(),
            }
        ];
        
        storage.save_apps(&apps, &key).unwrap();
        let loaded_apps = storage.load_apps(&key).unwrap();
        
        assert_eq!(apps.len(), loaded_apps.len());
        assert_eq!(apps[0].name, loaded_apps[0].name);
    }
}
