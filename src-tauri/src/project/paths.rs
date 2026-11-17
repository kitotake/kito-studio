//! Calcul de chemins portables (purement lexical : ne touche pas au disque).
//!
//! Les chemins relatifs stockés dans un `.kito` utilisent toujours `/` comme
//! séparateur, quel que soit le système, pour rester lisibles d'un ordinateur à l'autre.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Supprime `.` et résout `..` lexicalement (sans suivre les liens symboliques).
pub fn clean(path: &Path) -> PathBuf {
    let mut out: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match out.last() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir) | Some(Component::Prefix(_)) => {}
                _ => out.push(component),
            },
            other => out.push(other),
        }
    }
    if out.is_empty() {
        return PathBuf::from(".");
    }
    out.iter().map(|c| c.as_os_str()).collect()
}

fn same_name(a: &OsStr, b: &OsStr) -> bool {
    if cfg!(windows) {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

fn is_absolute_like(path: &Path) -> bool {
    path.has_root() || path.is_absolute()
}

/// Chemin de `target` relatif au dossier `base_dir` (séparateur `/`).
/// `None` si les deux chemins ne sont pas absolus ou sont sur des lecteurs différents.
pub fn to_relative(base_dir: &Path, target: &Path) -> Option<String> {
    if !is_absolute_like(base_dir) || !is_absolute_like(target) {
        return None;
    }
    let base = clean(base_dir);
    let target = clean(target);
    let b: Vec<Component> = base.components().collect();
    let t: Vec<Component> = target.components().collect();

    // Lecteur (préfixe Windows) différent : aucun chemin relatif possible.
    if let (Some(Component::Prefix(pb)), Some(Component::Prefix(pt))) = (b.first(), t.first()) {
        if !same_name(pb.as_os_str(), pt.as_os_str()) {
            return None;
        }
    } else if matches!(b.first(), Some(Component::Prefix(_)))
        != matches!(t.first(), Some(Component::Prefix(_)))
    {
        return None;
    }

    let mut common = 0;
    while common < b.len()
        && common < t.len()
        && same_name(b[common].as_os_str(), t[common].as_os_str())
    {
        common += 1;
    }

    let mut parts: Vec<String> = Vec::new();
    for component in &b[common..] {
        if matches!(component, Component::Normal(_)) {
            parts.push("..".to_string());
        }
    }
    for component in &t[common..] {
        if let Component::Normal(name) = component {
            parts.push(name.to_string_lossy().into_owned());
        }
    }
    if parts.is_empty() {
        return Some(".".to_string());
    }
    Some(parts.join("/"))
}

/// Résout un chemin relatif (séparateur `/`) par rapport à `base_dir`.
/// `None` si le chemin est vide, absolu ou contient un octet nul.
pub fn from_relative(base_dir: &Path, relative: &str) -> Option<PathBuf> {
    if relative.is_empty() || relative.contains('\0') {
        return None;
    }
    let rel = Path::new(relative);
    if relative.starts_with('/') || relative.starts_with('\\') || is_absolute_like(rel) {
        return None;
    }
    let mut joined = base_dir.to_path_buf();
    for part in relative.split('/').filter(|p| !p.is_empty()) {
        joined.push(part);
    }
    Some(clean(&joined))
}

pub fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn cleans_dots_lexically() {
        assert_eq!(clean(&p("/a/./b/../c")), p("/a/c"));
        assert_eq!(clean(&p("/a/../../b")), p("/b"));
        assert_eq!(clean(&p("a/../..")), p(".."));
        assert_eq!(clean(&p("")), p("."));
    }

    #[cfg(unix)]
    #[test]
    fn relative_paths_between_unix_dirs() {
        assert_eq!(
            to_relative(&p("/home/a/Projet"), &p("/home/a/Projet/media/clip.mp4")).as_deref(),
            Some("media/clip.mp4")
        );
        assert_eq!(
            to_relative(&p("/home/a/Projet"), &p("/home/a/Rushes/clip.mp4")).as_deref(),
            Some("../Rushes/clip.mp4")
        );
        assert_eq!(
            to_relative(&p("/home/a/Projet"), &p("/mnt/disk/clip.mp4")).as_deref(),
            Some("../../../mnt/disk/clip.mp4")
        );
        assert_eq!(to_relative(&p("/a/b"), &p("/a/b")).as_deref(), Some("."));
        assert_eq!(to_relative(&p("rel"), &p("/a")), None);
    }

    #[cfg(unix)]
    #[test]
    fn round_trips_through_from_relative() {
        let base = p("/home/a/Projet");
        let target = p("/home/a/Rushes/été 1/clip.mp4");
        let rel = to_relative(&base, &target).unwrap();
        assert_eq!(from_relative(&base, &rel), Some(target));
        // Le projet déplacé : le même relatif pointe ailleurs, comme voulu.
        assert_eq!(
            from_relative(&p("/mnt/usb/Projet"), &rel),
            Some(p("/mnt/usb/Rushes/été 1/clip.mp4"))
        );
    }

    #[test]
    fn rejects_absolute_or_invalid_relative_input() {
        assert_eq!(from_relative(&p("/base"), ""), None);
        assert_eq!(from_relative(&p("/base"), "/etc/passwd"), None);
        assert_eq!(from_relative(&p("/base"), "bad\0name"), None);
    }

    #[cfg(windows)]
    #[test]
    fn different_drives_have_no_relative_path() {
        assert_eq!(
            to_relative(Path::new(r"C:\Projets\A"), Path::new(r"D:\Rushes\a.mp4")),
            None
        );
        assert_eq!(
            to_relative(
                Path::new(r"C:\Projets\A"),
                Path::new(r"c:\projets\Rushes\a.mp4")
            )
            .as_deref(),
            Some("../Rushes/a.mp4")
        );
    }
}
