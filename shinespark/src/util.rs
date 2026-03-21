use std::{
    env,
    path::{Path, PathBuf},
};

fn is_workspace_root(path: &Path) -> bool {
    // 워크스페이스 루트임을 판별하는 기준 (예: 특정 파일 존재 여부)
    path.join("Cargo.lock").exists()
        || std::fs::read_to_string(path.join("Cargo.toml"))
            .map(|s| s.contains("[workspace]"))
            .unwrap_or(false)
}

pub fn workspace_root() -> PathBuf {
    // 1. 컴파일 시점의 경로 (Cargo.toml이 있는 곳)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 2. 해당 경로에 'Cargo.toml'이 있고, 그 안에 [workspace] 가 있는지 확인하며 상위로 이동
    // (간단하게는 특정 표식 파일이나 Cargo.lock이 있는 곳을 루트로 간주)
    let mut current = manifest_dir.as_path();
    while let Some(parent) = current.parent() {
        if parent.join("Cargo.toml").exists() {
            // 워크스페이스 루트에는 보통 Cargo.lock이나 멤버 폴더들이 있음
            if is_workspace_root(parent) {
                return parent.to_path_buf();
            }
            current = parent;
        } else {
            break;
        }
    }
    manifest_dir
}

/// 실행파일 -> 현재 작업 디렉토리로 fallback path 를 리턴한다.
pub fn base_path() -> PathBuf {
    if let Ok(mut exe_path) = env::current_exe() {
        exe_path.pop();
        return exe_path;
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn base_executable_name() -> String {
    // 1. 우선순위 1: 특정 바이너리 이름 (bin 타겟)
    // 2. 우선순위 2: 크레이트 이름 (lib/test 타겟)
    let base_name = option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_CRATE_NAME"));

    #[cfg(debug_assertions)]
    {
        // 개발/테스트 중에는 항상 "default" 혹은 "dev"처럼 고정된 이름을 반환하거나
        // 혹은 단순히 base_name을 그대로 써도 워크스페이스 루트에서 찾으므로 안전합니다.
        base_name.to_string()
    }

    #[cfg(not(debug_assertions))]
    {
        // 릴리즈 모드에서는 실행 파일의 실제 이름을 가져와서
        // 사용자가 파일명을 변경해서 배포해도 대응할 수 있게 합니다.
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem()?.to_str()?.to_owned().into())
            .unwrap_or_else(|| base_name.to_string())
    }
}
