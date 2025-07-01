use crate::{error::Error, Result};
use std::process::Command;

pub struct GitConfig;

impl GitConfig {
    pub fn is_git_repo() -> bool {
        Command::new("git")
            .args(&["rev-parse", "--git-dir"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn set_user_name(name: &str) -> Result<()> {
        let status = Command::new("git")
            .args(&["config", "--local", "user.name", name])
            .status()?;

        if !status.success() {
            return Err(Error::Git("user.name 설정 실패".to_string()));
        }
        Ok(())
    }

    pub fn set_user_email(email: &str) -> Result<()> {
        let status = Command::new("git")
            .args(&["config", "--local", "user.email", email])
            .status()?;

        if !status.success() {
            return Err(Error::Git("user.email 설정 실패".to_string()));
        }
        Ok(())
    }

    pub fn set_signing_key(key: &str) -> Result<()> {
        let status = Command::new("git")
            .args(&["config", "--local", "user.signingkey", key])
            .status()?;

        if !status.success() {
            return Err(Error::Git("user.signingkey 설정 실패".to_string()));
        }
        Ok(())
    }

    pub fn get_user_name() -> Result<String> {
        let output = Command::new("git")
            .args(&["config", "--local", "user.name"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(Error::Git("user.name을 가져올 수 없습니다".to_string()))
        }
    }

    pub fn get_user_email() -> Result<String> {
        let output = Command::new("git")
            .args(&["config", "--local", "user.email"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(Error::Git("user.email을 가져올 수 없습니다".to_string()))
        }
    }

    pub fn get_remotes() -> Result<Vec<(String, String)>> {
        let output = Command::new("git").args(&["remote", "-v"]).output()?;

        if !output.status.success() {
            return Err(Error::Git("리모트 정보를 가져올 수 없습니다".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut remotes = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[2] == "(fetch)" {
                remotes.push((parts[0].to_string(), parts[1].to_string()));
            }
        }

        Ok(remotes)
    }

    pub fn set_remote_url(remote_name: &str, url: &str) -> Result<()> {
        let status = Command::new("git")
            .args(&["remote", "set-url", remote_name, url])
            .status()?;

        if !status.success() {
            return Err(Error::Git(format!(
                "리모트 '{}' URL 설정 실패",
                remote_name
            )));
        }
        Ok(())
    }

    pub fn get_current_directory() -> Result<String> {
        std::env::current_dir()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| Error::Io(e))
    }

    /// Git Credential Manager에서 GitHub 계정 목록 조회
    pub fn get_cached_github_accounts() -> Result<Vec<String>> {
        // Windows에서만 credential-manager 사용
        if cfg!(windows) {
            let output = Command::new("git")
                .args(&["credential-manager", "github", "list"])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let accounts: Vec<String> = stdout
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect();
                Ok(accounts)
            } else {
                Ok(Vec::new())
            }
        } else {
            // Linux/macOS에서는 credential manager가 없으므로 빈 리스트 반환
            Ok(Vec::new())
        }
    }

    /// Git Credential Manager에서 특정 GitHub 계정 삭제
    pub fn clear_github_credentials(username: &str) -> Result<()> {
        if cfg!(windows) {
            let status = Command::new("git")
                .args(&["credential-manager", "github", "logout", username])
                .status()?;

            if status.success() {
                println!("  🔧 캐시된 크리덴셜 삭제: {}", username);
            } else {
                println!("  💡 크리덴셜이 캐시되지 않았음: {}", username);
            }
        } else {
            // Linux/macOS에서는 credential erase 사용
            Self::erase_credentials_for_host_linux("github.com", username)?;
        }

        Ok(())
    }

    /// 모든 GitHub 계정의 크리덴셜 삭제
    pub fn clear_all_github_credentials() -> Result<()> {
        if cfg!(windows) {
            let accounts = Self::get_cached_github_accounts()?;
            
            if accounts.is_empty() {
                println!("  💡 캐시된 GitHub 계정이 없습니다");
            } else {
                println!("  🔧 캐시된 GitHub 계정들을 삭제합니다: {:?}", accounts);
                for account in accounts {
                    Self::clear_github_credentials(&account)?;
                }
            }
        } else {
            println!("  💡 Linux/macOS에서는 개별 계정 삭제만 지원됩니다");
        }

        Ok(())
    }

    /// Linux/macOS용 크리덴셜 삭제
    fn erase_credentials_for_host_linux(host: &str, username: &str) -> Result<()> {
        use std::io::Write;
        
        let child = Command::new("git")
            .args(&["credential", "erase"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                if let Some(stdin) = child.stdin.as_mut() {
                    let input = format!("protocol=https\nhost={}\nusername={}\n", host, username);
                    
                    if let Err(_) = stdin.write_all(input.as_bytes()) {
                        println!("  💡 크리덴셜 삭제 입력 실패: {}@{}", username, host);
                        return Ok(());
                    }
                }

                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            println!("  🔧 크리덴셜 삭제 완료: {}@{}", username, host);
                        } else {
                            println!("  💡 크리덴셜이 저장되지 않았음: {}@{}", username, host);
                        }
                    }
                    Err(_) => {
                        println!("  💡 크리덴셜 삭제 명령 실패: {}@{}", username, host);
                    }
                }
            }
            Err(_) => {
                println!("  💡 git credential erase 명령 실행 실패: {}@{}", username, host);
            }
        }

        Ok(())
    }

    /// 특정 호스트의 크리덴셜 삭제 (범용) - 개선된 버전
    pub fn erase_credentials_for_host(host: &str, username: &str) -> Result<()> {
        Self::erase_credentials_for_host_linux(host, username)
    }

    /// PAT를 Git Credential Manager에 직접 저장
    pub fn store_github_credentials(username: &str, pat: &str) -> Result<()> {
        if cfg!(windows) {
            // Windows에서는 기존 방식 사용
            Self::store_credentials_windows(username, pat)
        } else {
            // Linux/WSL에서는 .git-credentials 파일 직접 생성
            Self::store_credentials_linux(username, pat)
        }
    }

    /// Windows용 credential 저장
    fn store_credentials_windows(username: &str, pat: &str) -> Result<()> {
        use std::io::Write;
        
        let child = Command::new("git")
            .args(&["credential", "store"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                if let Some(stdin) = child.stdin.as_mut() {
                    let input = format!(
                        "protocol=https\nhost=github.com\nusername={}\npassword={}\n",
                        username, pat
                    );
                    
                    if let Err(e) = stdin.write_all(input.as_bytes()) {
                        println!("  ⚠️  GitHub 크리덴셜 입력 실패: {} ({})", username, e);
                        return Ok(());
                    }
                }

                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            println!("  🔑 GitHub 크리덴셜 저장 완료: {}", username);
                        } else {
                            println!("  ⚠️  GitHub 크리덴셜 저장 실패: {} (exit code: {:?})", username, status.code());
                        }
                    }
                    Err(e) => {
                        println!("  ⚠️  GitHub 크리덴셜 저장 프로세스 대기 실패: {} ({})", username, e);
                    }
                }
            }
            Err(e) => {
                println!("  ⚠️  git credential store 명령 실행 실패: {} ({})", username, e);
            }
        }

        Ok(())
    }

    /// Linux/WSL용 credential 저장 - .git-credentials 파일 직접 생성
    fn store_credentials_linux(username: &str, pat: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::{BufRead, BufReader, Write};
        
        // credential.helper를 store로 설정
        let _ = Command::new("git")
            .args(&["config", "--global", "credential.helper", "store"])
            .status();

        // 홈 디렉토리의 .git-credentials 파일 경로
        let home_dir = dirs::home_dir()
            .ok_or_else(|| Error::Other("홈 디렉토리를 찾을 수 없습니다".to_string()))?;
        let credentials_file = home_dir.join(".git-credentials");

        // 기존 파일에서 동일한 host/username 항목 제거
        let mut existing_lines = Vec::new();
        if credentials_file.exists() {
            if let Ok(file) = std::fs::File::open(&credentials_file) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let line = line.trim();
                        if !line.is_empty() && 
                           !line.contains(&format!("://{}@github.com", username)) &&
                           !line.contains("github.com") {
                            existing_lines.push(line.to_string());
                        }
                    }
                }
            }
        }

        // 새로운 크리덴셜 추가
        let new_entry = format!("https://{}:{}@github.com", username, pat);
        existing_lines.push(new_entry);

        // 파일에 쓰기
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&credentials_file) 
        {
            Ok(mut file) => {
                for line in existing_lines {
                    if let Err(e) = writeln!(file, "{}", line) {
                        println!("  ⚠️  크리덴셜 파일 쓰기 실패: {} ({})", username, e);
                        return Ok(());
                    }
                }
                
                // 파일 권한을 600으로 설정 (보안)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&credentials_file, 
                        std::fs::Permissions::from_mode(0o600));
                }
                
                println!("  🔑 GitHub 크리덴셜 저장 완료: {}", username);
                println!("  📁 저장 위치: {}", credentials_file.display());
            }
            Err(e) => {
                println!("  ⚠️  크리덴셜 파일 생성 실패: {} ({})", username, e);
            }
        }

        Ok(())
    }

    /// GitHub PAT의 유효성을 간단히 테스트
    pub fn test_github_pat(_username: &str, pat: &str) -> Result<bool> {
        // GitHub API를 통해 토큰 유효성 검증
        let output = Command::new("curl")
            .args(&[
                "-s", "-o", "/dev/null", "-w", "%{http_code}",
                "-H", &format!("Authorization: token {}", pat),
                "-H", "User-Agent: git-switcher",
                "https://api.github.com/user"
            ])
            .output();

        match output {
            Ok(result) => {
                let status_code = String::from_utf8_lossy(&result.stdout);
                Ok(status_code.trim() == "200")
            }
            Err(_) => {
                // curl이 없으면 기본적인 형식 검증만 수행
                Ok(crate::crypto::TokenCrypto::validate_github_pat(pat))
            }
        }
    }
}
