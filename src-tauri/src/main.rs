#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    AppHandle, ClipboardManager, CustomMenuItem, Manager, Runtime, SystemTray, SystemTrayEvent, SystemTrayMenu,
};
use totp_rs::{Algorithm, TOTP};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OtpApp {
    id: String,
    name: String,
    secret: String,
}

struct AppState {
    apps: Mutex<Vec<OtpApp>>,
    master_password: Mutex<Option<String>>,
    encryption_key: Mutex<Option<[u8; 32]>>,
}

fn get_data_file_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push(".plaxo-otp");
    std::fs::create_dir_all(&path).unwrap_or_default();
    path.push("apps.enc");
    println!("Caminho do arquivo: {:?}", path);
    path
}

fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(b"plaxo-otp-salt-2024");
    hasher.finalize().into()
}

fn encrypt_data(data: &str, key: &[u8; 32]) -> Result<String, String> {
    use rand::RngCore;
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    // Gera nonce aleatório para cada criptografia
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|_| "Erro ao criptografar dados".to_string())?;
    
    // Combina nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(result))
}

fn decrypt_data(encrypted_data: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    let combined = general_purpose::STANDARD.decode(encrypted_data)
        .map_err(|_| "Dados corrompidos".to_string())?;
    
    if combined.len() < 12 {
        return Err("Dados corrompidos".to_string());
    }
    
    // Separa nonce + ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| "Senha incorreta ou dados corrompidos".to_string())?;
    
    String::from_utf8(plaintext).map_err(|_| "Dados corrompidos".to_string())
}

fn save_apps(apps: &[OtpApp], key: &[u8; 32]) -> Result<(), String> {
    let json = serde_json::to_string(apps).map_err(|_| "Erro ao serializar dados".to_string())?;
    let encrypted = encrypt_data(&json, key)?;
    let file_path = get_data_file_path();
    
    println!("Salvando {} apps em: {:?}", apps.len(), file_path);
    
    // Escrita segura com arquivo temporário
    let temp_path = format!("{}.tmp", file_path.to_string_lossy());
    
    // Backup do arquivo atual se existir
    if file_path.exists() {
        let backup_path = format!("{}.backup", file_path.to_string_lossy());
        fs::copy(&file_path, &backup_path).map_err(|e| format!("Erro ao criar backup: {}", e))?;
    }
    
    // Escreve no arquivo temporário
    fs::write(&temp_path, &encrypted).map_err(|e| format!("Erro ao escrever arquivo temporário: {}", e))?;
    
    // Move atomicamente para o arquivo final
    fs::rename(&temp_path, &file_path).map_err(|e| format!("Erro ao finalizar salvamento: {}", e))?;
    
    println!("Arquivo salvo com sucesso");
    Ok(())
}

fn load_apps(key: &[u8; 32]) -> Result<Vec<OtpApp>, String> {
    let file_path = get_data_file_path();
    
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    
    // Tenta carregar o arquivo principal
    match try_load_file(&file_path, key) {
        Ok(apps) => Ok(apps),
        Err(e) => {
            println!("Erro ao carregar arquivo principal: {}", e);
            
            // Tenta carregar do backup
            let backup_path = format!("{}.backup", file_path.to_string_lossy());
            if std::path::Path::new(&backup_path).exists() {
                println!("Tentando carregar do backup...");
                match try_load_file(&std::path::PathBuf::from(&backup_path), key) {
                    Ok(apps) => {
                        println!("Backup carregado com sucesso, restaurando arquivo principal...");
                        // Restaura o arquivo principal do backup
                        if let Err(restore_err) = fs::copy(&backup_path, &file_path) {
                            println!("Aviso: Não foi possível restaurar arquivo principal: {}", restore_err);
                        }
                        Ok(apps)
                    }
                    Err(backup_err) => {
                        println!("Backup também corrompido: {}", backup_err);
                        Err(e) // Retorna o erro original
                    }
                }
            } else {
                Err(e)
            }
        }
    }
}

fn try_load_file(file_path: &std::path::Path, key: &[u8; 32]) -> Result<Vec<OtpApp>, String> {
    let encrypted_data = fs::read_to_string(file_path).map_err(|_| "Erro ao ler arquivo".to_string())?;
    let decrypted = decrypt_data(&encrypted_data, key)?;
    let apps: Vec<OtpApp> = serde_json::from_str(&decrypted).map_err(|_| "Dados corrompidos".to_string())?;
    Ok(apps)
}

#[tauri::command]
fn has_master_password(state: tauri::State<AppState>) -> bool {
    let master_pass = state.master_password.lock().unwrap();
    if master_pass.is_some() {
        return true;
    }
    
    // Verifica se existe arquivo de dados criptografados
    let file_path = get_data_file_path();
    file_path.exists()
}

#[tauri::command]
fn copy_to_clipboard<R: Runtime>(app: AppHandle<R>, text: String) -> Result<(), String> {
    app.clipboard_manager()
        .write_text(text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn import_2fas_file(file_content: String, state: tauri::State<AppState>) -> Result<usize, String> {
    use serde_json::Value;
    
    println!("Iniciando importação do 2FAS...");
    
    let json: Value = serde_json::from_str(&file_content)
        .map_err(|_| "Arquivo 2FAS inválido".to_string())?;
    
    let services = json.get("services")
        .and_then(|s| s.as_array())
        .ok_or("Formato de arquivo 2FAS inválido".to_string())?;
    
    let mut apps = state.apps.lock().unwrap();
    let mut imported_count = 0;
    
    for service in services {
        if let (Some(name), Some(secret)) = (
            service.get("name").and_then(|n| n.as_str()),
            service.get("secret").and_then(|s| s.as_str())
        ) {
            let app = OtpApp {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.to_string(),
                secret: secret.to_string(),
            };
            apps.push(app);
            imported_count += 1;
            println!("Importado: {}", name);
        }
    }
    
    println!("Total importado: {}, Total na memória: {}", imported_count, apps.len());
    
    // Salva os dados criptografados
    let encryption_key = state.encryption_key.lock().unwrap();
    if let Some(key) = encryption_key.as_ref() {
        println!("Salvando dados importados...");
        save_apps(&apps, key)?;
    } else {
        println!("ERRO: Chave de criptografia não encontrada na importação!");
        return Err("Chave de criptografia não encontrada".to_string());
    }
    
    Ok(imported_count)
}

#[tauri::command]
fn verify_master_password(password: String, state: tauri::State<AppState>) -> bool {
    let mut master_pass = state.master_password.lock().unwrap();
    let mut encryption_key = state.encryption_key.lock().unwrap();
    
    let file_path = get_data_file_path();
    let file_exists = file_path.exists();
    
    if master_pass.is_none() && !file_exists {
        // Primeira vez - define a senha
        *master_pass = Some(password.clone());
        let key = derive_key(&password);
        *encryption_key = Some(key);
        println!("Primeira vez - senha definida");
        true
    } else if file_exists {
        // Arquivo existe - verifica senha e carrega dados
        let key = derive_key(&password);
        
        match load_apps(&key) {
            Ok(loaded_apps) => {
                *master_pass = Some(password);
                *encryption_key = Some(key);
                let mut apps = state.apps.lock().unwrap();
                *apps = loaded_apps;
                println!("Dados carregados: {} apps", apps.len());
                true
            }
            Err(e) => {
                println!("Erro ao carregar dados: {}", e);
                false
            }
        }
    } else {
        // Verifica senha na sessão atual
        master_pass.as_ref() == Some(&password)
    }
}

#[tauri::command]
fn get_apps(state: tauri::State<AppState>) -> Vec<OtpApp> {
    state.apps.lock().unwrap().clone()
}

#[tauri::command]
fn add_app(name: String, secret: String, state: tauri::State<AppState>) -> Result<(), String> {
    println!("Adicionando app: {}", name);
    let id = uuid::Uuid::new_v4().to_string();
    let app = OtpApp { id, name, secret };
    
    let mut apps = state.apps.lock().unwrap();
    apps.push(app);
    println!("Total de apps na memória: {}", apps.len());
    
    // Salva os dados criptografados
    let encryption_key = state.encryption_key.lock().unwrap();
    if let Some(key) = encryption_key.as_ref() {
        println!("Salvando dados criptografados...");
        save_apps(&apps, key)?;
    } else {
        println!("ERRO: Chave de criptografia não encontrada!");
        return Err("Chave de criptografia não encontrada".to_string());
    }
    
    Ok(())
}

#[tauri::command]
fn delete_app(id: String, state: tauri::State<AppState>) -> Result<(), String> {
    println!("Deletando app com ID: {}", id);
    let mut apps = state.apps.lock().unwrap();
    let before_count = apps.len();
    apps.retain(|app| app.id != id);
    let after_count = apps.len();
    println!("Apps antes: {}, depois: {}", before_count, after_count);
    
    // Salva os dados criptografados
    let encryption_key = state.encryption_key.lock().unwrap();
    if let Some(key) = encryption_key.as_ref() {
        println!("Salvando após deletar...");
        save_apps(&apps, key)?;
    } else {
        println!("ERRO: Chave de criptografia não encontrada na deleção!");
        return Err("Chave de criptografia não encontrada".to_string());
    }
    
    Ok(())
}

#[tauri::command]
fn reset_master_password(state: tauri::State<AppState>) -> Result<(), String> {
    println!("Resetando senha mestre e dados...");
    
    // Limpa estado na memória
    {
        let mut master_pass = state.master_password.lock().unwrap();
        let mut encryption_key = state.encryption_key.lock().unwrap();
        let mut apps = state.apps.lock().unwrap();
        
        *master_pass = None;
        *encryption_key = None;
        apps.clear();
    }
    
    // Remove arquivos do disco
    let file_path = get_data_file_path();
    let backup_path = format!("{}.backup", file_path.to_string_lossy());
    
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| format!("Erro ao remover arquivo principal: {}", e))?;
        println!("Arquivo principal removido");
    }
    
    if std::path::Path::new(&backup_path).exists() {
        fs::remove_file(&backup_path).map_err(|e| format!("Erro ao remover backup: {}", e))?;
        println!("Backup removido");
    }
    
    println!("Reset concluído com sucesso");
    Ok(())
}

#[tauri::command]
fn generate_otp(app_id: String, state: tauri::State<AppState>) -> Result<String, String> {
    let apps = state.apps.lock().unwrap();
    let app = apps.iter().find(|a| a.id == app_id)
        .ok_or("App não encontrado".to_string())?;
    
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        app.secret.as_bytes().to_vec(),
    ).map_err(|e| e.to_string())?;
    
    let code = totp.generate_current().map_err(|e| e.to_string())?;
    Ok(code)
}

fn create_tray() -> SystemTray {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            let window = app.get_window("main").unwrap();
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "open" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn main() {
    let state = AppState {
        apps: Mutex::new(Vec::new()),
        master_password: Mutex::new(None),
        encryption_key: Mutex::new(None),
    };

    tauri::Builder::<tauri::Wry>::new()
        .manage(state)
        .system_tray(create_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            has_master_password,
            copy_to_clipboard,
            import_2fas_file,
            verify_master_password,
            reset_master_password,
            get_apps,
            add_app,
            delete_app,
            generate_otp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
