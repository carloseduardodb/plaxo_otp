use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::{AppError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleDriveAuth {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct FileResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    files: Vec<FileResponse>,
}

pub struct GoogleDriveClient {
    client: Client,
    client_id: String,
    client_secret: String,
}

impl GoogleDriveClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            client_id: "1035416240731-ggg4vbs5grbklttmr9qt9jq645mbt3eq.apps.googleusercontent.com".to_string(),
            client_secret: "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".to_string(),
        }
    }

    pub fn get_auth_url(&self) -> String {
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri=http://localhost:8080&response_type=code&scope=https://www.googleapis.com/auth/drive.file&access_type=offline",
            self.client_id
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<GoogleDriveAuth> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", "http://localhost:8080");

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleDrive(format!("HTTP {}: {}", status, error_text)));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Invalid response: {}", e)))?;

        Ok(GoogleDriveAuth {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            expires_at: chrono::Utc::now().timestamp() as u64 + token_response.expires_in,
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleDriveAuth> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("refresh_token", refresh_token);
        params.insert("grant_type", "refresh_token");

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Refresh request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleDrive(format!("Refresh failed HTTP {}: {}", status, error_text)));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Invalid refresh response: {}", e)))?;

        Ok(GoogleDriveAuth {
            access_token: token_response.access_token,
            refresh_token: refresh_token.to_string(),
            expires_at: chrono::Utc::now().timestamp() as u64 + token_response.expires_in,
        })
    }

    pub async fn find_file(&self, auth: &GoogleDriveAuth, filename: &str) -> Result<Option<String>> {
        tracing::info!("Searching for file: {}", filename);
        
        let response = self.client
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(&auth.access_token)
            .query(&[("q", format!("name='{}'", filename))])
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Search request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleDrive(format!("Search failed HTTP {}: {}", status, error_text)));
        }

        let files_response: FilesResponse = response
            .json()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Invalid search response: {}", e)))?;

        if let Some(file) = files_response.files.first() {
            tracing::info!("File found! ID: {}", file.id);
            Ok(Some(file.id.clone()))
        } else {
            tracing::info!("File not found");
            Ok(None)
        }
    }

    pub async fn upload_file(&self, auth: &GoogleDriveAuth, filename: &str, content: &[u8]) -> Result<String> {
        tracing::info!("Uploading file: {} ({} bytes)", filename, content.len());
        
        let metadata = serde_json::json!({
            "name": filename
        });

        let form = reqwest::multipart::Form::new()
            .part("metadata", reqwest::multipart::Part::text(metadata.to_string())
                .mime_str("application/json")
                .map_err(|e| AppError::GoogleDrive(format!("Failed to create metadata: {}", e)))?)
            .part("media", reqwest::multipart::Part::bytes(content.to_vec())
                .file_name(filename.to_string())
                .mime_str("text/plain")
                .map_err(|e| AppError::GoogleDrive(format!("Failed to create media: {}", e)))?);

        let response = self.client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(&auth.access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Upload request failed: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| AppError::GoogleDrive(format!("Failed to read response: {}", e)))?;
        
        if !status.is_success() {
            return Err(AppError::GoogleDrive(format!("Upload failed HTTP {}: {}", status, response_text)));
        }

        let file_response: FileResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError::GoogleDrive(format!("Invalid upload response: {} - Response: {}", e, response_text)))?;

        tracing::info!("Upload completed! File ID: {}", file_response.id);
        Ok(file_response.id)
    }

    pub async fn update_file(&self, auth: &GoogleDriveAuth, file_id: &str, content: &[u8]) -> Result<()> {
        tracing::info!("Updating file ID: {} ({} bytes)", file_id, content.len());
        
        let response = self.client
            .patch(&format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media", file_id))
            .bearer_auth(&auth.access_token)
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Update request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleDrive(format!("Update failed HTTP {}: {}", status, error_text)));
        }

        tracing::info!("File updated successfully!");
        Ok(())
    }

    pub async fn download_file(&self, auth: &GoogleDriveAuth, file_id: &str) -> Result<Vec<u8>> {
        tracing::info!("Downloading file ID: {}", file_id);
        
        let response = self.client
            .get(&format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id))
            .bearer_auth(&auth.access_token)
            .send()
            .await
            .map_err(|e| AppError::GoogleDrive(format!("Download request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleDrive(format!("Download failed HTTP {}: {}", status, error_text)));
        }

        let bytes = response.bytes().await
            .map_err(|e| AppError::GoogleDrive(format!("Failed to read download: {}", e)))?
            .to_vec();

        tracing::info!("Downloaded {} bytes", bytes.len());
        Ok(bytes)
    }
}

impl Default for GoogleDriveClient {
    fn default() -> Self {
        Self::new()
    }
}
