use std::path::PathBuf;

pub fn path_from_string(path: &str, script_path: &PathBuf) -> Option<PathBuf> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        return Some(path);
    }
    if let Some(p) = script_path
        .canonicalize()
        .unwrap_or_else(|_| script_path.clone())
        .parent()
    {
        #[cfg(debug_assertions)]
        eprintln!("path: parent path: {p:?}");
        let p = p.join(&path);
        #[cfg(debug_assertions)]
        eprintln!("path: joined: {p:?}");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(mers_lib_dir) = std::env::var("MERS_LIB_DIR") {
        let p = PathBuf::from(mers_lib_dir).join(&path);
        if p.exists() {
            return Some(p);
        }
    }
    None
}
