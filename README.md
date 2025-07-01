# Git-Switcher

Git 프로필 전환을 자동화하는 CLI 도구입니다. 회사/개인 계정 간 Git 설정을 쉽게 관리하고 전환할 수 있습니다.

## 주요 기능

- **프로필 관리**: 여러 Git 계정 정보를 프로필로 저장
- **자동 전환**: 디렉토리 기반 프로필 자동 감지 및 적용
- **크리덴셜 관리**: GitHub PAT 암호화 저장 및 자동 설정
- **SSH/HTTPS 지원**: SSH 키 또는 HTTPS 방식 모두 지원
- **충돌 방지**: 기존 크리덴셜 자동 정리로 계정 충돌 방지

## 설치

```bash
# Rust 설치 후
cargo install --path .

# 또는 직접 빌드
cargo build --release
```

## 빠른 시작

### 1. 초기 설정
```bash
git-switcher init
```

### 2. 프로필 추가
```bash
# 기본 프로필 (HTTPS 방식)
git-switcher add work \
  --user-name "홍길동" \
  --email "hong@company.com" \
  --github-username "hong-work"

# PAT 포함 프로필 (자동 크리덴셜 설정)
git-switcher add personal \
  --user-name "홍길동" \
  --email "hong@gmail.com" \
  --github-username "hong-personal" \
  --github-pat "ghp_xxxxxxxxxxxx"
```

### 3. 프로필 사용
```bash
# 수동 적용
git-switcher use work

# 자동 감지 (경로 매핑 기반)
git-switcher auto

# 경로 매핑 설정
git-switcher map ~/work work
git-switcher map ~/personal personal
```

## 명령어 참조

### 프로필 관리
```bash
git-switcher list                    # 프로필 목록
git-switcher add <name> [options]    # 프로필 추가
git-switcher remove <name>           # 프로필 제거
git-switcher show                    # 현재 프로필 확인
```

### 프로필 적용
```bash
git-switcher use <name>              # 프로필 적용
git-switcher use <name> --ssh        # SSH 모드로 적용
git-switcher auto                    # 자동 감지 적용
```

### 경로 매핑
```bash
git-switcher map <path> <profile>    # 경로-프로필 매핑 추가
git-switcher unmap <path>            # 매핑 제거
```

### 크리덴셜 관리
```bash
git-switcher credentials list        # 캐시된 계정 목록
git-switcher credentials clear <user> # 특정 계정 삭제
git-switcher credentials clear-all   # 모든 계정 삭제
```

## 설정 파일 구조

`~/.config/git-switcher/config.toml`:

```toml
[profiles.work]
name = "홍길동"
email = "hong@company.com"
github_username = "hong-work"
encrypted_pat = "base64_encrypted_token"
signingkey = "GPG_KEY_ID"
ssh_key = "~/.ssh/id_ed25519_work"

[profiles.personal]
name = "홍길동"
email = "hong@gmail.com"
github_username = "hong-personal"
encrypted_pat = "base64_encrypted_token"

[path_mappings]
"/home/user/work" = "work"
"/home/user/personal" = "personal"
```

## 아키텍처

### 모듈 구조
```
src/
├── core/          # 핵심 비즈니스 로직
│   ├── config.rs  # 설정 파일 관리
│   ├── profile.rs # 프로필 매니저
│   └── error.rs   # 에러 타입
├── git/           # Git 관련 기능
│   ├── operations.rs    # Git 명령어 래퍼
│   └── credentials.rs   # 크리덴셜 관리
└── utils/         # 유틸리티
    ├── crypto.rs  # 토큰 암호화
    ├── ssh.rs     # SSH 연동
    └── auto.rs    # 자동 감지
```

### 보안 기능

- **PAT 암호화**: AES-256-GCM으로 토큰 암호화 저장
- **빌드별 키**: 컴파일 시마다 다른 암호화 키 생성
- **토큰 마스킹**: 로그에서 토큰 내용 숨김
- **권한 관리**: 설정 파일 권한 600으로 제한

## 사용 시나리오

### 회사/개인 프로젝트 분리
```bash
# 회사 프로젝트 디렉토리
cd ~/work/company-repo
git-switcher use work
git commit -m "회사 계정으로 커밋"

# 개인 프로젝트 디렉토리  
cd ~/personal/my-project
git-switcher use personal
git commit -m "개인 계정으로 커밋"
```

### 자동화된 워크플로우
```bash
# 경로 매핑 설정 후
git-switcher map ~/work work
git-switcher map ~/personal personal

# 자동 적용
cd ~/work/any-project
git-switcher auto  # work 프로필 자동 적용

cd ~/personal/any-project  
git-switcher auto  # personal 프로필 자동 적용
```

## 문제 해결

### 크리덴셜 충돌
```bash
# 기존 크리덴셜 모두 정리
git-switcher credentials clear-all

# 프로필 재적용
git-switcher use <profile-name>
```

### PAT 관련 문제
```bash
# PAT 유효성 확인
curl -H "Authorization: token YOUR_PAT" https://api.github.com/user

# 새 PAT로 프로필 업데이트
git-switcher add <profile-name> --github-pat <new-pat>
```

### SSH 설정 문제
SSH 키가 있는 경우 `~/.ssh/config` 설정 확인:
```
Host github.com-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_ed25519_work
```

## 기여

이슈나 PR은 언제든 환영합니다. 