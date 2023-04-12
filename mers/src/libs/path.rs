use std::path::PathBuf;

pub fn path_from_string(path: &str, script_directory: &PathBuf) -> Option<PathBuf> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        return Some(path);
    }
    if let Some(p) = script_directory
        .canonicalize()
        .unwrap_or_else(|_| script_directory.clone())
        .parent()
    {
        eprintln!("Parent: {:?}", p);
        let p = p.join(&path);
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
