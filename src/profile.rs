use crate::{config::Profile, error::Error, git::GitConfig, Result};

pub struct ProfileManager;

impl ProfileManager {
    pub fn apply_profile(profile: &Profile, enable_ssh: bool) -> Result<()> {
        if !GitConfig::is_git_repo() {
            return Err(Error::NotGitRepo);
        }

        // 기본 Git 설정 적용
        GitConfig::set_user_name(&profile.name)?;
        GitConfig::set_user_email(&profile.email)?;

        // GPG 서명 키 설정 (선택사항)
        if let Some(signing_key) = &profile.signingkey {
            GitConfig::set_signing_key(signing_key)?;
        }

        // SSH 연동 (선택사항)
        if enable_ssh && profile.ssh_key.is_some() {
            crate::ssh::SshManager::configure_remotes_for_profile(profile)?;
        } else {
            // HTTPS URL에 사용자명 포함 (SSH 없는 경우)
            Self::configure_https_remotes_for_profile(profile)?;
        }

        println!("✓ 프로필이 적용되었습니다.");
        println!("  이름: {}", profile.name);
        println!("  이메일: {}", profile.email);

        if let Some(key) = &profile.signingkey {
            println!("  GPG 키: {}", key);
        }

        if enable_ssh && profile.ssh_key.is_some() {
            println!("  SSH 연동: 활성화됨");
        }

        Ok(())
    }

    pub fn show_current_profile() -> Result<()> {
        if !GitConfig::is_git_repo() {
            return Err(Error::NotGitRepo);
        }

        match (GitConfig::get_user_name(), GitConfig::get_user_email()) {
            (Ok(name), Ok(email)) => {
                println!("현재 Git 프로필:");
                println!("  이름: {}", name);
                println!("  이메일: {}", email);
            }
            _ => {
                println!("현재 저장소에 Git 프로필이 설정되어 있지 않습니다.");
            }
        }

        Ok(())
    }

    fn configure_https_remotes_for_profile(profile: &Profile) -> Result<()> {
        let remotes = GitConfig::get_remotes()?;

        for (remote_name, url) in remotes {
            if let Some(new_url) = Self::transform_url_for_https(&url, profile) {
                GitConfig::set_remote_url(&remote_name, &new_url)?;
                println!("  {} 리모트 URL 변경: {} -> {}", remote_name, url, new_url);
            }
        }

        Ok(())
    }

    fn transform_url_for_https(url: &str, profile: &Profile) -> Option<String> {
        // GitHub/GitLab HTTPS URL에 사용자명 추가
        if url.starts_with("https://github.com/") && !url.contains("@") {
            // github_username이 있으면 사용, 없으면 URL에서 추출
            let username = if let Some(gh_username) = &profile.github_username {
                gh_username.clone()
            } else {
                Self::extract_username_from_url(url)?
            };
            let repo_path = url.replace("https://github.com/", "");
            return Some(format!("https://{}@github.com/{}", username, repo_path));
        }

        if url.starts_with("https://gitlab.com/") && !url.contains("@") {
            let username = if let Some(gh_username) = &profile.github_username {
                gh_username.clone()
            } else {
                Self::extract_username_from_url(url)?
            };
            let repo_path = url.replace("https://gitlab.com/", "");
            return Some(format!("https://{}@gitlab.com/{}", username, repo_path));
        }

        None
    }

    fn extract_username_from_url(url: &str) -> Option<String> {
        // URL에서 실제 사용자명 추출
        // 예: https://github.com/whoisdreamer/blog → whoisdreamer
        if let Some(path_start) = url.find(".com/") {
            let path = &url[path_start + 5..]; // ".com/" 이후
            if let Some(slash_pos) = path.find('/') {
                return Some(path[..slash_pos].to_string());
            }
        }
        None
    }
}
