use std::{
    env,
    path::{Path, PathBuf},
};

/// 워크스페이스 디렉토리를 반환하는 함수입니다.
///
/// # 주의
/// `CARGO_MANIFEST_DIR`의 상위 경로를 참조하므로 실제 워크스페이스와 다를 수 있습니다.
///
pub fn workspace_dir() -> PathBuf {
    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_dir() {
        let workspace_dir = workspace_dir();
        assert!(workspace_dir.ends_with("shinespark"));
    }

    #[test]
    fn test_executable_path() {
        let executable_path = env::current_dir().unwrap().to_str().unwrap().to_string();
        println!("Executable path: {}", executable_path);
    }
}
