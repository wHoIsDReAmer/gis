use clap::Parser;
use git_switcher::{
    auto::AutoDetector,
    cli::{Cli, Commands},
    config::{Config, Profile},
    profile::ProfileManager,
    ssh::SshManager,
    Result,
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
                    let config_path = git_switcher::config::get_config_path()?;
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
            signing_key,
            ssh_key,
        } => {
            let mut config = Config::load().unwrap_or_default();

            let profile = Profile {
                name: user_name,
                email,
                signingkey: signing_key,
                ssh_key,
                github_username,
            };

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
    }

    Ok(())
}
