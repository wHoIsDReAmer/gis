use crate::{error::Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub path_mappings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signingkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_username: Option<String>,
    /// 암호화된 GitHub Personal Access Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_pat: Option<String>,
}

impl Profile {
    /// PAT가 설정되어 있는지 확인
    pub fn has_pat(&self) -> bool {
        self.encrypted_pat.is_some()
    }
    
    /// 암호화된 PAT를 복호화해서 반환
    pub fn get_decrypted_pat(&self) -> Result<Option<String>> {
        if let Some(encrypted_pat) = &self.encrypted_pat {
            let decrypted = crate::crypto::TokenCrypto::decrypt_token(encrypted_pat)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }
    
    /// PAT를 암호화해서 저장
    pub fn set_encrypted_pat(&mut self, pat: &str) -> Result<()> {
        let encrypted = crate::crypto::TokenCrypto::encrypt_token(pat)?;
        self.encrypted_pat = Some(encrypted);
        Ok(())
    }
    
    /// 안전하게 마스킹된 PAT 정보 표시
    pub fn get_masked_pat(&self) -> Option<String> {
        if let Ok(Some(pat)) = self.get_decrypted_pat() {
            Some(crate::crypto::TokenCrypto::mask_token(&pat))
        } else {
            None
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Err(Error::ConfigNotFound);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile> {
        self.profiles
            .get(name)
            .ok_or_else(|| Error::ProfileNotFound(name.to_string()))
    }

    pub fn add_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<()> {
        if self.profiles.remove(name).is_none() {
            return Err(Error::ProfileNotFound(name.to_string()));
        }
        Ok(())
    }

    pub fn add_path_mapping(&mut self, path: String, profile: String) {
        self.path_mappings.insert(path, profile);
    }

    pub fn remove_path_mapping(&mut self, path: &str) -> bool {
        self.path_mappings.remove(path).is_some()
    }

    pub fn find_profile_for_path(&self, current_path: &str) -> Option<&String> {
        let mut best_match: Option<(&String, &String)> = None;
        let mut best_length = 0;

        for (mapped_path, profile) in &self.path_mappings {
            let expanded_path = expand_path(mapped_path);
            if current_path.starts_with(&expanded_path) && expanded_path.len() > best_length {
                best_match = Some((mapped_path, profile));
                best_length = expanded_path.len();
            }
        }

        best_match.map(|(_, profile)| profile)
    }

    pub fn init_default() -> Result<Self> {
        let mut profiles = HashMap::new();

        profiles.insert(
            "personal".to_string(),
            Profile {
                name: "Your Name".to_string(),
                email: "your.email@personal.com".to_string(),
                signingkey: None,
                ssh_key: None,
                github_username: Some("your-github-username".to_string()),
                encrypted_pat: None,
            },
        );

        profiles.insert(
            "company".to_string(),
            Profile {
                name: "Your Name".to_string(),
                email: "your.email@company.com".to_string(),
                signingkey: None,
                ssh_key: None,
                github_username: Some("your-work-username".to_string()),
                encrypted_pat: None,
            },
        );

        let mut path_mappings = HashMap::new();
        path_mappings.insert("~/workspace/personal/".to_string(), "personal".to_string());
        path_mappings.insert("~/workspace/company/".to_string(), "company".to_string());

        let config = Config {
            profiles,
            path_mappings,
        };
        config.save()?;
        Ok(config)
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| Error::Other("홈 디렉토리를 찾을 수 없습니다".to_string()))?;
    let config_dir = home_dir.join(".config").join("git-switcher");
    Ok(config_dir.join("config.toml"))
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replace("~", &home.to_string_lossy());
        }
    }
    path.to_string()
}
