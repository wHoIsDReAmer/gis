use crate::{config::Profile, git::GitConfig, Result};
use std::collections::HashMap;

pub struct SshManager;

impl SshManager {
    pub fn configure_remotes_for_profile(profile: &Profile) -> Result<()> {
        let remotes = GitConfig::get_remotes()?;

        for (remote_name, url) in remotes {
            if let Some(new_url) = Self::transform_url_for_ssh(&url, profile) {
                GitConfig::set_remote_url(&remote_name, &new_url)?;
                println!("  {} 리모트 URL 변경: {} -> {}", remote_name, url, new_url);
            }
        }

        Ok(())
    }

    fn transform_url_for_ssh(url: &str, profile: &Profile) -> Option<String> {
        // SSH URL 변환 로직
        if let Some(ssh_key) = &profile.ssh_key {
            let ssh_config_name = Self::extract_ssh_config_name(ssh_key);

            // GitHub SSH URL 변환
            if url.contains("github.com") {
                if url.starts_with("git@github.com:") {
                    // 이미 SSH 형식인 경우
                    return Some(
                        url.replace("github.com", &format!("github.com-{}", ssh_config_name)),
                    );
                } else if url.starts_with("https://github.com/") {
                    // HTTPS를 SSH로 변환
                    let repo_path = url.replace("https://github.com/", "");
                    let repo_path = repo_path.trim_end_matches(".git");
                    return Some(format!(
                        "git@github.com-{}:{}.git",
                        ssh_config_name, repo_path
                    ));
                }
            }

            // GitLab SSH URL 변환
            if url.contains("gitlab.com") {
                if url.starts_with("git@gitlab.com:") {
                    return Some(
                        url.replace("gitlab.com", &format!("gitlab.com-{}", ssh_config_name)),
                    );
                } else if url.starts_with("https://gitlab.com/") {
                    let repo_path = url.replace("https://gitlab.com/", "");
                    let repo_path = repo_path.trim_end_matches(".git");
                    return Some(format!(
                        "git@gitlab.com-{}:{}.git",
                        ssh_config_name, repo_path
                    ));
                }
            }
        }

        None
    }

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
