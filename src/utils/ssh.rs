use crate::core::Profile;
use std::collections::HashMap;

pub struct SshManager;

impl SshManager {
    fn extract_ssh_config_name(ssh_key_path: &str) -> String {
        // SSH 키 경로에서 설정 이름 추출
        // 예: ~/.ssh/id_ed25519_personal -> personal
        if let Some(filename) = ssh_key_path.split('/').last() {
            if let Some(name) = filename.strip_prefix("id_ed25519_") {
                return name.to_string();
            }
            if let Some(name) = filename.strip_prefix("id_rsa_") {
                return name.to_string();
            }
        }

        // 기본값으로 키 파일명 사용
        ssh_key_path
            .split('/')
            .last()
            .unwrap_or("default")
            .to_string()
    }

    pub fn generate_ssh_config_example(profiles: &HashMap<String, Profile>) -> String {
        let mut config = String::new();
        config.push_str("# ~/.ssh/config 파일에 추가할 설정 예시\n\n");

        for (profile_name, profile) in profiles {
            if let Some(ssh_key) = &profile.ssh_key {
                let config_name = Self::extract_ssh_config_name(ssh_key);

                config.push_str(&format!("# {} 프로필용 GitHub 설정\n", profile_name));
                config.push_str(&format!("Host github.com-{}\n", config_name));
                config.push_str("    HostName github.com\n");
                config.push_str("    User git\n");
                config.push_str(&format!("    IdentityFile {}\n", ssh_key));
                config.push_str("    IdentitiesOnly yes\n\n");

                config.push_str(&format!("# {} 프로필용 GitLab 설정\n", profile_name));
                config.push_str(&format!("Host gitlab.com-{}\n", config_name));
                config.push_str("    HostName gitlab.com\n");
                config.push_str("    User git\n");
                config.push_str(&format!("    IdentityFile {}\n", ssh_key));
                config.push_str("    IdentitiesOnly yes\n\n");
            }
        }

        config
    }
}
