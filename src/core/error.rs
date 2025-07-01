use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Config(toml::de::Error),
    SerdeToml(toml::ser::Error),
    Git(String),
    NotGitRepo,
    ProfileNotFound(String),
    ConfigNotFound,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO 에러: {}", err),
            Error::Config(err) => write!(f, "설정 파일 파싱 에러: {}", err),
            Error::SerdeToml(err) => write!(f, "TOML 직렬화 에러: {}", err),
            Error::Git(msg) => write!(f, "Git 에러: {}", msg),
            Error::NotGitRepo => write!(f, "현재 디렉토리는 Git 저장소가 아닙니다"),
            Error::ProfileNotFound(name) => write!(f, "프로필 '{}'을 찾을 수 없습니다", name),
            Error::ConfigNotFound => write!(
                f,
                "설정 파일이 없습니다. 'git-switcher init' 명령으로 초기화하세요"
            ),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::SerdeToml(err)
    }
}
