use image::DynamicImage;
use rqrr::PreparedImage;

use crate::types::{AppError, Result};

pub struct QrCodeReader;

impl QrCodeReader {
    pub fn new() -> Self {
        Self
    }

    pub fn decode_from_image(&self, image_data: &[u8]) -> Result<String> {
        let img = image::load_from_memory(image_data)
            .map_err(|_| AppError::QrCode("Invalid image format".to_string()))?;
        
        self.decode_from_dynamic_image(img)
    }

    fn decode_from_dynamic_image(&self, img: DynamicImage) -> Result<String> {
        let gray_img = img.to_luma8();
        let mut prepared = PreparedImage::prepare(gray_img);
        let grids = prepared.detect_grids();
        
        if grids.is_empty() {
            return Err(AppError::QrCode("No QR code found in image".to_string()));
        }
        
        let (_, content) = grids[0]
            .decode()
            .map_err(|_| AppError::QrCode("Failed to decode QR code".to_string()))?;
        
        if content.starts_with("otpauth://totp/") {
            self.extract_secret_from_otpauth(&content)
        } else {
            Ok(content)
        }
    }

    fn extract_secret_from_otpauth(&self, content: &str) -> Result<String> {
        if !content.starts_with("otpauth://totp/") {
            return Err(AppError::QrCode("Not a valid OTP QR code".to_string()));
        }
        
        let secret_start = content
            .find("secret=")
            .ok_or_else(|| AppError::QrCode("QR code does not contain secret key".to_string()))?;
        
        let secret_part = &content[secret_start + 7..];
        let secret = if let Some(secret_end) = secret_part.find('&') {
            &secret_part[..secret_end]
        } else {
            secret_part
        };
        
        if secret.is_empty() {
            return Err(AppError::QrCode("Empty secret in QR code".to_string()));
        }
        
        Ok(secret.to_string())
    }


}

impl Default for QrCodeReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_secret_from_otpauth() {
        let reader = QrCodeReader::new();
        let otpauth = "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example";
        
        let secret = reader.extract_secret_from_otpauth(otpauth).unwrap();
        assert_eq!(secret, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn test_extract_name_from_otpauth() {
        let reader = QrCodeReader::new();
        let otpauth = "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example";
        
        let name = reader.extract_name_from_otpauth(otpauth).unwrap();
        assert_eq!(name, "Example:alice@google.com");
    }

    #[test]
    fn test_invalid_otpauth() {
        let reader = QrCodeReader::new();
        let invalid = "not an otpauth url";
        
        let result = reader.extract_secret_from_otpauth(invalid);
        assert!(result.is_err());
    }
}
