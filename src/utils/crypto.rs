use crate::core::{Error, Result};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};

/// 컴파일 시간 기반 고유 키 생성
fn get_build_key() -> [u8; 32] {
    // 컴파일 시간과 버전 정보를 조합해서 키 생성
    let build_info = format!(
        "{}{}{}{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        file!(),
        line!()
    );

    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(build_info.as_bytes());
    hasher.update(b"git-switcher-secret");

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    key
}

pub struct TokenCrypto;

impl TokenCrypto {
    /// PAT를 암호화하여 저장 가능한 문자열로 변환
    pub fn encrypt_token(token: &str) -> Result<String> {
        let key = get_build_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| Error::Other(format!("암호화 키 생성 실패: {}", e)))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, token.as_bytes())
            .map_err(|e| Error::Other(format!("토큰 암호화 실패: {}", e)))?;

        // nonce와 ciphertext를 합쳐서 base64로 인코딩
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(encrypted_data))
    }

    /// 암호화된 토큰을 복호화
    pub fn decrypt_token(encrypted_token: &str) -> Result<String> {
        let key = get_build_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| Error::Other(format!("암호화 키 생성 실패: {}", e)))?;

        let encrypted_data = general_purpose::STANDARD
            .decode(encrypted_token)
            .map_err(|e| Error::Other(format!("토큰 디코딩 실패: {}", e)))?;

        if encrypted_data.len() < 12 {
            return Err(Error::Other("잘못된 암호화 데이터 형식".to_string()));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::Other(format!("토큰 복호화 실패: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| Error::Other(format!("토큰 문자열 변환 실패: {}", e)))
    }

    /// 토큰의 유효성 검증 (기본적인 PAT 형식 체크)
    pub fn validate_github_pat(token: &str) -> bool {
        // GitHub PAT 형식: ghp_, gho_, ghu_, ghs_, ghr_ 등으로 시작
        token.starts_with("ghp_")
            || token.starts_with("gho_")
            || token.starts_with("ghu_")
            || token.starts_with("ghs_")
            || token.starts_with("ghr_")
            || token.starts_with("github_pat_")
    }

    /// 토큰을 마스킹해서 안전하게 표시
    pub fn mask_token(token: &str) -> String {
        if token.len() <= 8 {
            "*".repeat(token.len())
        } else {
            let prefix = &token[..4];
            let suffix = &token[token.len() - 4..];
            format!("{}***{}", prefix, suffix)
        }
    }
}
