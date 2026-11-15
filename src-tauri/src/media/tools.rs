//! Détection des binaires FFmpeg / FFprobe.

use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolsStatus {
    pub ffmpeg_found: bool,
    pub ffprobe_found: bool,
}

/// Nom du fichier exécutable selon la plateforme (`ffmpeg` → `ffmpeg.exe` sous Windows).
pub fn executable_file_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Cherche `name` dans une liste de dossiers et retourne le premier fichier trouvé.
pub fn find_in_dirs<I, P>(dirs: I, name: &str) -> Option<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let file_name = executable_file_name(name);
    dirs.into_iter()
        .map(|dir| dir.as_ref().join(&file_name))
        .find(|candidate| candidate.is_file())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    find_in_dirs(env::split_paths(&paths), name)
}

pub fn detect() -> MediaToolsStatus {
    MediaToolsStatus {
        ffmpeg_found: find_in_path("ffmpeg").is_some(),
        ffprobe_found: find_in_path("ffprobe").is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Dossier temporaire unique, supprimé à la fin du test.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = env::temp_dir().join(format!("kito-test-{label}-{}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn finds_an_existing_executable() {
        let dir = TempDir::new("found");
        fs::write(dir.0.join(executable_file_name("fake-tool")), b"").unwrap();
        let found = find_in_dirs([&dir.0], "fake-tool");
        assert_eq!(found, Some(dir.0.join(executable_file_name("fake-tool"))));
    }

    #[test]
    fn returns_none_when_missing() {
        let dir = TempDir::new("missing");
        assert_eq!(find_in_dirs([&dir.0], "fake-tool"), None);
    }

    #[test]
    fn ignores_directories_with_the_same_name() {
        let dir = TempDir::new("dir");
        fs::create_dir(dir.0.join(executable_file_name("fake-tool"))).unwrap();
        assert_eq!(find_in_dirs([&dir.0], "fake-tool"), None);
    }
}
