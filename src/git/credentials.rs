use crate::core::{Profile, Result};
use crate::git::GitConfig;

pub struct CredentialManager;

impl CredentialManager {
    /// PAT 크리덴셜 자동 설정
    pub fn setup_pat_credentials(profile: &Profile) -> Result<()> {
        if let Some(github_username) = &profile.github_username
            && let Ok(Some(pat)) = profile.get_decrypted_pat()
        {
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
        Ok(())
    }

    /// 기존 크리덴셜 삭제하여 계정 충돌 방지
    pub fn clear_existing_credentials(profile: &Profile) -> Result<()> {
        // GitHub 계정의 경우
        if let Some(github_username) = &profile.github_username {
            // 1. Git Credential Manager에서 GitHub 계정 삭제 시도
            GitConfig::clear_github_credentials(github_username)?;

            // 2. 일반 크리덴셜 삭제도 시도 (Windows/Linux 호환성)
            let _ = GitConfig::erase_credentials_for_host("github.com", github_username);
        }

        // URL에서 추출한 사용자명으로도 시도
        let remotes = GitConfig::get_remotes()?;
        for (_, url) in &remotes {
            if let Some(username) = Self::extract_username_from_url(url) {
                if username != profile.github_username.as_deref().unwrap_or("") {
                    GitConfig::clear_github_credentials(&username)?;
                    let _ = GitConfig::erase_credentials_for_host("github.com", &username);
                }
            }
        }

        Ok(())
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
