use clap::Parser;
use git_switcher::{
    cli::{Cli, Commands, CredentialAction},
    core::{Config, Profile, ProfileManager, Result},
    git::GitConfig,
    utils::{auto::AutoDetector, ssh::SshManager, crypto::TokenCrypto},
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Use { profile_name, ssh } => {
            let config = Config::load()?;
            let profile = config.get_profile(&profile_name)?;
            ProfileManager::apply_profile(profile, ssh)?;
        }

        Commands::Show => {
            ProfileManager::show_current_profile()?;
        }

        Commands::List => {
            let config = Config::load()?;

            if config.profiles.is_empty() {
                println!("등록된 프로필이 없습니다.");
            } else {
                println!("사용 가능한 프로필:");
                for (name, profile) in &config.profiles {
                    print!("  {}: {} <{}>", name, profile.name, profile.email);
                    if profile.ssh_key.is_some() {
                        print!(" [SSH]");
                    }
                    if profile.has_pat() {
                        if let Some(masked) = profile.get_masked_pat() {
                            print!(" [PAT: {}]", masked);
                        }
                    }
                    println!();
                }
            }

            if !config.path_mappings.is_empty() {
                println!("\n경로 매핑:");
                for (path, profile) in &config.path_mappings {
                    println!("  {} -> {}", path, profile);
                }
            }
        }

        Commands::Init => {
            match Config::init_default() {
                Ok(_) => {
                    let config_path = git_switcher::core::config::get_config_path()?;
                    println!("✓ 설정 파일이 생성되었습니다: {}", config_path.display());
                    println!("설정 파일을 편집하여 프로필을 수정하세요.");

                    // SSH 설정 예시 출력
                    let config = Config::load()?;
                    if config.profiles.values().any(|p| p.ssh_key.is_some()) {
                        println!("\n🔧 SSH 설정 예시:");
                        println!(
                            "{}",
                            SshManager::generate_ssh_config_example(&config.profiles)
                        );
                    }
                }
                Err(e) => {
                    eprintln!("❌ 초기화 실패: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Auto { ssh } => {
            AutoDetector::detect_and_apply_profile(ssh)?;
        }

        Commands::Add {
            name,
            user_name,
            email,
            github_username,
            github_pat,
            signing_key,
            ssh_key,
        } => {
            let mut config = Config::load().unwrap_or_default();

            let mut profile = Profile {
                name: user_name,
                email,
                signingkey: signing_key,
                ssh_key,
                github_username: github_username.clone(),
                encrypted_pat: None,
            };

            // GitHub PAT 처리
            if let Some(pat) = github_pat {
                // PAT 유효성 검증
                if !TokenCrypto::validate_github_pat(&pat) {
                    println!("⚠️  경고: 입력된 토큰이 올바른 GitHub PAT 형식이 아닐 수 있습니다.");
                }

                // PAT를 암호화해서 저장
                match profile.set_encrypted_pat(&pat) {
                    Ok(_) => {
                        println!("🔑 GitHub PAT가 암호화되어 저장되었습니다: {}", TokenCrypto::mask_token(&pat));
                    }
                    Err(e) => {
                        eprintln!("❌ PAT 암호화 실패: {}", e);
                        std::process::exit(1);
                    }
                }

                // GitHub 사용자명이 없으면 PAT 검증을 통해 가져오기 시도
                if github_username.is_none() {
                    println!("💡 GitHub API를 통해 사용자명 확인 중...");
                    if let Ok(detected_username) = detect_github_username_from_pat(&pat) {
                        profile.github_username = Some(detected_username.clone());
                        println!("✓ GitHub 사용자명 자동 감지: {}", detected_username);
                    }
                }
            } else if github_username.is_some() {
                println!("💡 GitHub PAT를 나중에 추가하려면:");
                println!("  git-switcher add {} --github-pat <YOUR_PAT> (기존 프로필 업데이트)", name);
            }

            config.add_profile(name.clone(), profile);
            config.save()?;

            println!("✓ 프로필 '{}'이 추가되었습니다.", name);
        }

        Commands::Remove { name } => {
            let mut config = Config::load()?;
            config.remove_profile(&name)?;
            config.save()?;

            println!("✓ 프로필 '{}'이 제거되었습니다.", name);
        }

        Commands::Map { path, profile } => {
            let mut config = Config::load()?;

            // 프로필이 존재하는지 확인
            config.get_profile(&profile)?;

            config.add_path_mapping(path.clone(), profile.clone());
            config.save()?;

            println!("✓ 경로 매핑이 추가되었습니다: {} -> {}", path, profile);
        }

        Commands::Unmap { path } => {
            let mut config = Config::load()?;

            if config.remove_path_mapping(&path) {
                config.save()?;
                println!("✓ 경로 매핑이 제거되었습니다: {}", path);
            } else {
                println!("❌ 해당 경로의 매핑을 찾을 수 없습니다: {}", path);
            }
        }

        Commands::Credentials { action } => {
            match action {
                CredentialAction::List => {
                    println!("🔍 캐시된 GitHub 계정들:");
                    match GitConfig::get_cached_github_accounts() {
                        Ok(accounts) => {
                            if accounts.is_empty() {
                                println!("  (캐시된 계정이 없습니다)");
                            } else {
                                for account in accounts {
                                    println!("  - {}", account);
                                }
                            }
                        }
                        Err(e) => {
                            println!("  ❌ 계정 목록 조회 실패: {}", e);
                        }
                    }
                }

                CredentialAction::Clear { username } => {
                    println!("🔧 계정 '{}' 크리덴셜 삭제 중...", username);
                    GitConfig::clear_github_credentials(&username)?;
                    let _ = GitConfig::erase_credentials_for_host("github.com", &username);
                    println!("✓ 계정 '{}' 크리덴셜이 삭제되었습니다.", username);
                }

                CredentialAction::ClearAll => {
                    println!("🔧 모든 GitHub 계정 크리덴셜 삭제 중...");
                    GitConfig::clear_all_github_credentials()?;
                    println!("✓ 모든 GitHub 계정 크리덴셜이 삭제되었습니다.");
                }
            }
        }
    }

    Ok(())
}

/// GitHub API를 통해 PAT에서 사용자명 추출
fn detect_github_username_from_pat(pat: &str) -> Result<String> {
    use std::process::Command;
    
    let output = Command::new("curl")
        .args(&[
            "-s",
            "-H", &format!("Authorization: token {}", pat),
            "-H", "User-Agent: git-switcher",
            "https://api.github.com/user"
        ])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let response = String::from_utf8_lossy(&result.stdout);
                // JSON에서 login 필드 추출 (간단한 파싱)
                if let Some(start) = response.find("\"login\":\"") {
                    let start = start + 9;
                    if let Some(end) = response[start..].find("\"") {
                        return Ok(response[start..start+end].to_string());
                    }
                }
            }
            Err(git_switcher::core::Error::Other("GitHub API 응답 파싱 실패".to_string()))
        }
        Err(_) => {
            Err(git_switcher::core::Error::Other("curl 명령 실행 실패".to_string()))
        }
    }
}
