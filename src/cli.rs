use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "git-switcher")]
#[command(about = "Git 사용자 프로필 전환 도구")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 프로필을 현재 저장소에 적용
    Use {
        /// 적용할 프로필 이름
        profile_name: String,
        /// SSH 연동 활성화
        #[arg(long)]
        ssh: bool,
    },
    /// 현재 적용된 프로필 확인
    Show,
    /// 사용 가능한 프로필 목록
    List,
    /// 초기 설정 파일 생성
    Init,
    /// 자동으로 프로필 감지하여 적용
    Auto {
        /// SSH 연동 활성화
        #[arg(long)]
        ssh: bool,
    },
    /// 프로필 추가
    Add {
        /// 프로필 이름
        name: String,
        /// 사용자 이름
        #[arg(long)]
        user_name: String,
        /// 이메일 주소
        #[arg(long)]
        email: String,
        /// GitHub 사용자명 (선택사항)
        #[arg(long)]
        github_username: Option<String>,
        /// GPG 서명 키 (선택사항)
        #[arg(long)]
        signing_key: Option<String>,
        /// SSH 키 경로 (선택사항)
        #[arg(long)]
        ssh_key: Option<String>,
    },
    /// 프로필 제거
    Remove {
        /// 제거할 프로필 이름
        name: String,
    },
    /// 경로 매핑 추가 (자동 감지용)
    Map {
        /// 디렉토리 경로
        path: String,
        /// 매핑할 프로필 이름
        profile: String,
    },
    /// 경로 매핑 제거
    Unmap {
        /// 제거할 디렉토리 경로
        path: String,
    },
}
