use crate::{
    core::{Config, Error, ProfileManager, Result},
    git::GitConfig,
};

pub struct AutoDetector;

impl AutoDetector {
    pub fn detect_and_apply_profile(enable_ssh: bool) -> Result<()> {
        if !GitConfig::is_git_repo() {
            return Err(Error::NotGitRepo);
        }

        let config = Config::load()?;
        let current_path = GitConfig::get_current_directory()?;

        if let Some(profile_name) = config.find_profile_for_path(&current_path) {
            let profile = config.get_profile(profile_name)?;
            println!("🔍 자동 감지된 프로필: {}", profile_name);
            println!("📂 경로: {}", current_path);

            ProfileManager::apply_profile(profile, enable_ssh)?;

            Ok(())
        } else {
            println!("❌ 현재 경로에 매핑된 프로필이 없습니다.");
            println!("📂 현재 경로: {}", current_path);
            println!();
            println!("사용 가능한 경로 매핑:");

            if config.path_mappings.is_empty() {
                println!("  (매핑된 경로가 없습니다)");
                println!();
                println!("경로 매핑을 추가하려면:");
                println!("  git-switcher map <경로> <프로필>");
                println!("  예: git-switcher map ~/workspace/company company");
            } else {
                for (path, profile) in &config.path_mappings {
                    println!("  {} -> {}", path, profile);
                }
            }

            Err(Error::Other("매핑된 프로필이 없습니다".to_string()))
        }
    }

    pub fn find_git_repos_in_mapped_paths(config: &Config) -> Result<Vec<(String, String)>> {
        let mut repos = Vec::new();

        for (mapped_path, profile_name) in &config.path_mappings {
            let expanded_path = expand_path(mapped_path);

            if let Ok(entries) = std::fs::read_dir(&expanded_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join(".git").exists() {
                        repos.push((path.to_string_lossy().to_string(), profile_name.clone()));
                    }
                }
            }
        }

        Ok(repos)
    }

    pub fn apply_to_all_mapped_repos(enable_ssh: bool) -> Result<()> {
        let config = Config::load()?;
        let repos = Self::find_git_repos_in_mapped_paths(&config)?;

        if repos.is_empty() {
            println!("매핑된 경로에서 Git 저장소를 찾을 수 없습니다.");
            return Ok(());
        }

        println!("🔍 발견된 Git 저장소들:");
        for (repo_path, profile_name) in &repos {
            println!("  {} -> {}", repo_path, profile_name);
        }
        println!();

        let current_dir = std::env::current_dir()?;

        for (repo_path, profile_name) in repos {
            println!("📂 처리 중: {}", repo_path);

            // 디렉토리 변경
            if let Err(e) = std::env::set_current_dir(&repo_path) {
                println!("  ❌ 디렉토리 변경 실패: {}", e);
                continue;
            }

            // 프로필 적용
            match config.get_profile(&profile_name) {
                Ok(profile) => match ProfileManager::apply_profile(profile, enable_ssh) {
                    Ok(_) => println!("  ✓ {} 프로필 적용 완료", profile_name),
                    Err(e) => println!("  ❌ 프로필 적용 실패: {}", e),
                },
                Err(e) => println!("  ❌ 프로필 로드 실패: {}", e),
            }

            println!();
        }

        // 원래 디렉토리로 복귀
        std::env::set_current_dir(current_dir)?;

        Ok(())
    }
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replace("~", &home.to_string_lossy());
        }
    }
    path.to_string()
}
