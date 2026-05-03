use std::path::{Path, PathBuf};

/// Returns the config directory to use.
///
/// If `binary_dir/config.toml` exists on the filesystem, returns `binary_dir`
/// (portable/sidecar mode — config lives beside the binary).
/// Otherwise returns `platform_dir` (the standard platform-appropriate config
/// directory provided by the caller via the `directories` crate).
pub fn resolve_config_path(binary_dir: &Path, platform_dir: &Path) -> PathBuf {
    if binary_dir.join("config.toml").exists() {
        binary_dir.to_path_buf()
    } else {
        platform_dir.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn returns_binary_dir_when_config_exists_beside_binary() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("config.toml"), "[settings]").unwrap();
        let platform = std::path::PathBuf::from("/platform/config/dir");
        assert_eq!(resolve_config_path(tmp.path(), &platform), tmp.path());
    }

    #[test]
    fn returns_platform_dir_when_no_sidecar_config() {
        let tmp = tempfile::tempdir().unwrap();
        // No config.toml written
        let platform = std::path::PathBuf::from("/platform/config/dir");
        assert_eq!(resolve_config_path(tmp.path(), &platform), platform);
    }
}
