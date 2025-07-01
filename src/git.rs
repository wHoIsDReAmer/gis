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
}
