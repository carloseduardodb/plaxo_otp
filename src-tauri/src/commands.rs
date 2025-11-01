use std::io::prelude::*;
use std::net::TcpListener;

use auto_launch::AutoLaunchBuilder;
use tauri::{AppHandle, ClipboardManager, Runtime};

use crate::crypto::derive_key;
use crate::google_drive::GoogleDriveClient;
use crate::otp::OtpGenerator;
use crate::qr::QrCodeReader;
use crate::state::AppState;
use crate::storage::Storage;
use crate::sync::SyncManager;
use crate::types::{OtpApp, AppError, Result};

#[tauri::command]
pub fn has_master_password(state: tauri::State<AppState>) -> bool {
    if state.has_master_password() {
        return true;
    }
    
    // Check if encrypted data file exists
    let storage = Storage::new();
    storage.has_apps_file()
}

#[tauri::command]
pub fn verify_master_password(password: String, state: tauri::State<AppState>) -> bool {
    let storage = Storage::new();
    let file_exists = storage.has_apps_file();
    
    if !state.has_master_password() && !file_exists {
        // First time - set password
        state.set_master_password(password.clone());
        let key = derive_key(&password);
        state.set_encryption_key(key);
        tracing::info!("First time - password set");
        true
    } else if file_exists {
        // File exists - verify password and load data
        let key = derive_key(&password);
        
        match storage.load_apps(&key) {
            Ok(loaded_apps) => {
                state.set_master_password(password);
                state.set_encryption_key(key);
                state.set_apps(loaded_apps);
                tracing::info!("Data loaded: {} apps", state.get_apps().len());
                true
            }
            Err(e) => {
                tracing::error!("Failed to load data: {}", e);
                false
            }
        }
    } else {
        // Verify password in current session
        false // This case shouldn't happen in the current flow
    }
}

#[tauri::command]
pub fn get_apps(state: tauri::State<AppState>) -> Vec<OtpApp> {
    state.get_apps()
}

#[tauri::command]
pub fn add_app(name: String, secret: String, state: tauri::State<AppState>) -> Result<()> {
    tracing::info!("Adding app: {}", name);
    
    // Validate secret first
    let otp_generator = OtpGenerator::new();
    otp_generator.validate_secret(&secret)?;
    
    let id = uuid::Uuid::new_v4().to_string();
    let app = OtpApp { id, name, secret };
    
    state.add_app(app);
    tracing::info!("Total apps in memory: {}", state.get_apps().len());
    
    // Save encrypted data
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    
    let storage = Storage::new();
    storage.save_apps(&state.get_apps(), &key)?;
    
    Ok(())
}

#[tauri::command]
pub fn edit_app_name(id: String, new_name: String, state: tauri::State<AppState>) -> Result<()> {
    if !state.update_app_name(&id, new_name) {
        return Err(AppError::AppNotFound);
    }
    
    // Save encrypted data
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    
    let storage = Storage::new();
    storage.save_apps(&state.get_apps(), &key)?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_app(app_id: String, state: tauri::State<AppState>) -> Result<()> {
    tracing::info!("Deleting app with ID: {}", app_id);
    
    if !state.remove_app(&app_id) {
        return Err(AppError::AppNotFound);
    }
    
    tracing::info!("Apps after deletion: {}", state.get_apps().len());
    
    // Save encrypted data
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    
    let storage = Storage::new();
    storage.save_apps(&state.get_apps(), &key)?;
    
    Ok(())
}

#[tauri::command]
pub fn generate_otp(app_id: String, state: tauri::State<AppState>) -> Result<String> {
    let app = state.get_app_by_id(&app_id)
        .ok_or(AppError::AppNotFound)?;
    
    let otp_generator = OtpGenerator::new();
    otp_generator.generate_code(&app.secret)
}

#[tauri::command]
pub fn copy_to_clipboard<R: Runtime>(app: AppHandle<R>, text: String) -> Result<()> {
    app.clipboard_manager()
        .write_text(text)
        .map_err(|e| AppError::Io(e.to_string()))
}

#[tauri::command]
pub fn import_2fas_file(file_content: String, state: tauri::State<AppState>) -> Result<usize> {
    use serde_json::Value;
    
    tracing::info!("Starting 2FAS import...");
    
    let json: Value = serde_json::from_str(&file_content)?;
    
    let services = json.get("services")
        .and_then(|s| s.as_array())
        .ok_or_else(|| AppError::Serialization("Invalid 2FAS file format".to_string()))?;
    
    let mut imported_count = 0;
    let otp_generator = OtpGenerator::new();
    
    for service in services {
        if let (Some(name), Some(secret)) = (
            service.get("name").and_then(|n| n.as_str()),
            service.get("secret").and_then(|s| s.as_str())
        ) {
            // Validate secret
            if otp_generator.validate_secret(secret).is_ok() {
                let app = OtpApp {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: name.to_string(),
                    secret: secret.to_string(),
                };
                state.add_app(app);
                imported_count += 1;
                tracing::info!("Imported: {}", name);
            } else {
                tracing::warn!("Skipped invalid secret for: {}", name);
            }
        } else {
            tracing::warn!("Skipped service - missing required fields: {:?}", service);
        }
    }
    
    tracing::info!("Total imported: {}, Total in memory: {}", imported_count, state.get_apps().len());
    
    // Save encrypted data
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    
    let storage = Storage::new();
    storage.save_apps(&state.get_apps(), &key)?;
    
    Ok(imported_count)
}

#[tauri::command]
pub fn decode_qr_from_image(image_data: Vec<u8>) -> Result<String> {
    let qr_reader = QrCodeReader::new();
    qr_reader.decode_from_image(&image_data)
}

#[tauri::command]
pub fn decode_qr_from_clipboard() -> Result<String> {
    Err(AppError::QrCode("Please paste the QR code string manually or use an external QR code reader".to_string()))
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<()> {
    let exe_path = std::env::current_exe()
        .map_err(AppError::from)?;
    let exe_str = exe_path.to_str()
        .ok_or_else(|| AppError::Io("Invalid executable path".to_string()))?;
    
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name("Plaxo OTP")
        .set_app_path(exe_str)
        .build()
        .map_err(|e| AppError::Io(e.to_string()))?;

    if enabled {
        auto_launch.enable()
            .map_err(|e| AppError::Io(e.to_string()))?;
        tracing::info!("Autostart enabled");
    } else {
        auto_launch.disable()
            .map_err(|e| AppError::Io(e.to_string()))?;
        tracing::info!("Autostart disabled");
    }

    Ok(())
}

#[tauri::command]
pub fn get_autostart_status() -> Result<bool> {
    let exe_path = std::env::current_exe()
        .map_err(AppError::from)?;
    let exe_str = exe_path.to_str()
        .ok_or_else(|| AppError::Io("Invalid executable path".to_string()))?;
    
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name("Plaxo OTP")
        .set_app_path(exe_str)
        .build()
        .map_err(|e| AppError::Io(e.to_string()))?;

    auto_launch.is_enabled()
        .map_err(|e| AppError::Io(e.to_string()))
}

#[tauri::command]
pub fn reset_master_password(state: tauri::State<AppState>) -> Result<()> {
    tracing::info!("Resetting master password and data...");
    
    // Clear state in memory
    state.clear_all();
    
    // Remove files from disk
    let storage = Storage::new();
    storage.reset_all_data()?;
    
    tracing::info!("Reset completed successfully");
    Ok(())
}

// Google Drive commands
#[tauri::command]
pub async fn google_drive_auth_url() -> String {
    let client = GoogleDriveClient::new();
    client.get_auth_url()
}

#[tauri::command]
pub async fn google_drive_auth_flow(state: tauri::State<'_, AppState>) -> Result<()> {
    let client = GoogleDriveClient::new();
    let auth_url = client.get_auth_url();
    
    // Start local server on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080")
        .map_err(|_| AppError::GoogleDrive("Failed to start local server".to_string()))?;
    
    // Open URL in browser
    std::process::Command::new("xdg-open")
        .arg(&auth_url)
        .spawn()
        .map_err(|_| AppError::GoogleDrive("Failed to open browser".to_string()))?;
    
    // Wait for connection
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer);
                
                if let Some(code_start) = request.find("code=") {
                    let code_part = &request[code_start + 5..];
                    if let Some(code_end) = code_part.find(' ') {
                        let code = &code_part[..code_end];
                        
                        // HTTP response
                        let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authorization completed!</h1><p>You can close this window.</p></body></html>";
                        stream.write_all(response.as_bytes()).unwrap();
                        
                        // Exchange code for token
                        let auth = client.exchange_code(code).await?;
                        
                        // Save auth permanently
                        let key = state.get_encryption_key()
                            .ok_or(AppError::NoMasterPassword)?;
                        
                        let sync_manager = SyncManager::new();
                        sync_manager.save_google_auth(&auth, &key).await?;
                        
                        state.set_google_auth(Some(auth));
                        
                        tracing::info!("Authentication successful!");
                        return Ok(());
                    }
                }
            }
            Err(_) => continue,
        }
    }
    
    Err(AppError::GoogleDrive("Authentication timeout".to_string()))
}

#[tauri::command]
pub async fn sync_with_google_drive(state: tauri::State<'_, AppState>) -> Result<()> {
    let apps = state.get_apps();
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    let auth = state.get_google_auth()
        .ok_or_else(|| AppError::GoogleDrive("Google Drive not authenticated".to_string()))?;
    
    let sync_manager = SyncManager::new();
    sync_manager.sync_to_google_drive(&apps, &key, &auth).await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_from_google_drive(state: tauri::State<'_, AppState>) -> Result<usize> {
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    let auth = state.get_google_auth()
        .ok_or_else(|| AppError::GoogleDrive("Google Drive not authenticated".to_string()))?;
    
    let sync_manager = SyncManager::new();
    let cloud_apps = sync_manager.sync_from_google_drive(&key, &auth).await?;
    let count = cloud_apps.len();
    
    state.set_apps(cloud_apps);
    
    // Save locally too
    let storage = Storage::new();
    storage.save_apps(&state.get_apps(), &key)?;
    
    Ok(count)
}

#[tauri::command]
pub async fn check_google_auth(state: tauri::State<'_, AppState>) -> Result<bool> {
    // Check if already syncing
    if state.is_syncing() {
        tracing::info!("Sync already in progress, skipping...");
        return Ok(false);
    }
    
    state.set_syncing(true);
    
    let key = state.get_encryption_key()
        .ok_or(AppError::NoMasterPassword)?;
    
    let result = {
        let sync_manager = SyncManager::new();
        match sync_manager.load_google_auth(&key).await {
            Ok(auth) => {
                state.set_google_auth(Some(auth.clone()));
                
                // Initial sync - check if there's data in the cloud
                tracing::info!("Checking cloud data...");
                match sync_manager.sync_from_google_drive(&key, &auth).await {
                    Ok(cloud_apps) => {
                        if !cloud_apps.is_empty() {
                            tracing::info!("Found {} apps in cloud, syncing...", cloud_apps.len());
                            let mut current_apps = state.get_apps();
                            
                            // Merge: keep local apps and add new ones from cloud
                            for cloud_app in cloud_apps {
                                if !current_apps.iter().any(|local_app| local_app.id == cloud_app.id) {
                                    current_apps.push(cloud_app);
                                }
                            }
                            
                            state.set_apps(current_apps);
                            
                            // Save locally
                            let storage = Storage::new();
                            storage.save_apps(&state.get_apps(), &key)?;
                            tracing::info!("Initial sync completed!");
                        } else {
                            tracing::info!("No cloud data, uploading local data...");
                            let apps = state.get_apps();
                            if !apps.is_empty() {
                                sync_manager.sync_to_google_drive(&apps, &key, &auth).await?;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error checking cloud: {}", e);
                        // Don't fail if can't access cloud
                    }
                }
                
                Ok(true)
            }
            Err(_) => Ok(false)
        }
    };
    
    // Release the lock
    state.set_syncing(false);
    
    result
}

#[tauri::command]
pub async fn clear_google_auth(state: tauri::State<'_, AppState>) -> Result<()> {
    // Clear from memory
    state.set_google_auth(None);
    
    // Remove file from disk
    let storage = Storage::new();
    storage.clear_google_auth()?;
    
    tracing::info!("Google Drive authentication removed");
    Ok(())
}
