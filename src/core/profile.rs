use crate::core::{Error, Profile, Result};
use crate::git::{CredentialManager, GitConfig};

pub struct ProfileManager;

impl ProfileManager {
    pub fn apply_profile(profile: &Profile, enable_ssh: bool) -> Result<()> {
        if !GitConfig::is_git_repo() {
            return Err(Error::NotGitRepo);
        }

        // 1. 기존 크리덴셜 삭제 (계정 충돌 방지)
        println!("🔧 기존 크리덴셜 정리 중...");
        CredentialManager::clear_existing_credentials(profile)?;

        // 2. 기본 Git 설정 적용
        GitConfig::set_user_name(&profile.name)?;
        GitConfig::set_user_email(&profile.email)?;

        // 3. GPG 서명 키 설정 (선택사항)
        if let Some(signing_key) = &profile.signingkey {
            GitConfig::set_signing_key(signing_key)?;
        }

        if !enable_ssh && profile.has_pat() {
            // 5. PAT가 있으면 자동으로 크리덴셜 설정
            CredentialManager::setup_pat_credentials(profile)?;
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
}
