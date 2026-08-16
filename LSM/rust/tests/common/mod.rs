use std::fs;

pub fn delete_files_in_dir(dir: &str) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            delete_files_in_dir(&path.to_string_lossy().into_owned())?;
        }
    }
    Ok(())
}
