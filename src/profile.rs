use crate::{config::Profile, error::Error, git::GitConfig, Result};

pub struct ProfileManager;

impl ProfileManager {
    pub fn apply_profile(profile: &Profile, enable_ssh: bool) -> Result<()> {
        if !GitConfig::is_git_repo() {
            return Err(Error::NotGitRepo);
        }

        // 1. 기존 크리덴셜 삭제 (계정 충돌 방지)
        println!("🔧 기존 크리덴셜 정리 중...");
        Self::clear_existing_credentials(profile)?;

        // 2. 기본 Git 설정 적용
        GitConfig::set_user_name(&profile.name)?;
        GitConfig::set_user_email(&profile.email)?;

        // 3. GPG 서명 키 설정 (선택사항)
        if let Some(signing_key) = &profile.signingkey {
            GitConfig::set_signing_key(signing_key)?;
        }

        // 4. SSH 연동 또는 HTTPS URL 설정
        if enable_ssh && profile.ssh_key.is_some() {
            crate::ssh::SshManager::configure_remotes_for_profile(profile)?;
        } else {
            // HTTPS URL에 사용자명 포함 (SSH 없는 경우)
            Self::configure_https_remotes_for_profile(profile)?;
            
            // 5. PAT가 있으면 자동으로 크리덴셜 설정
            Self::setup_pat_credentials(profile)?;
        }

        println!("✓ 프로필이 적용되었습니다.");
        println!("  이름: {}", profile.name);
        println!("  이메일: {}", profile.email);

        if let Some(key) = &profile.signingkey {
            println!("  GPG 키: {}", key);
        }

        if enable_ssh && profile.ssh_key.is_some() {
            println!("  SSH 연동: 활성화됨");
        } else if profile.has_pat() {
            if let Some(masked_pat) = profile.get_masked_pat() {
                println!("  🔑 GitHub PAT: {}", masked_pat);
                println!("  💡 PAT가 자동으로 설정되었습니다. push가 바로 가능합니다!");
            }
        } else {
            println!("  💡 다음 push 시 새로운 PAT 입력이 필요합니다");
        }

        Ok(())
    }

    /// PAT 크리덴셜 자동 설정
    fn setup_pat_credentials(profile: &Profile) -> Result<()> {
        if let Some(github_username) = &profile.github_username {
            if let Ok(Some(pat)) = profile.get_decrypted_pat() {
                println!("  🔑 GitHub PAT 자동 설정 중...");
                
                // PAT 유효성 검증
                match GitConfig::test_github_pat(github_username, &pat) {
                    Ok(true) => {
                        GitConfig::store_github_credentials(github_username, &pat)?;
                    }
                    Ok(false) => {
                        println!("  ⚠️  PAT가 유효하지 않을 수 있습니다. 수동으로 확인해주세요.");
                        GitConfig::store_github_credentials(github_username, &pat)?;
                    }
                    Err(_) => {
                        // 검증 실패해도 저장은 시도
                        GitConfig::store_github_credentials(github_username, &pat)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 기존 크리덴셜 삭제하여 계정 충돌 방지
    fn clear_existing_credentials(profile: &Profile) -> Result<()> {
        // GitHub 계정의 경우
        if let Some(github_username) = &profile.github_username {
            // 1. Git Credential Manager에서 GitHub 계정 삭제 시도
            GitConfig::clear_github_credentials(github_username)?;
            
            // 2. 일반 크리덴셜 삭제도 시도 (Windows/Linux 호환성)
            let _ = GitConfig::erase_credentials_for_host("github.com", github_username);
        }

        // URL에서 추출한 사용자명으로도 시도
        let remotes = GitConfig::get_remotes()?;
        for (_, url) in remotes {
            if let Some(username) = Self::extract_username_from_url(&url) {
                if username != profile.github_username.as_deref().unwrap_or("") {
                    GitConfig::clear_github_credentials(&username)?;
                    let _ = GitConfig::erase_credentials_for_host("github.com", &username);
                }
            }
        }

        // 안전을 위해 모든 GitHub 계정 삭제 (옵션)
        // GitConfig::clear_all_github_credentials()?;

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
