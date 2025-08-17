use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use dirs_next as dirs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MuscleMapping {
    pub primary: String,
    pub secondary: Vec<String>,
    pub category: String,
}

static MAPPINGS: Lazy<Mutex<HashMap<String, MuscleMapping>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const FILE: &str = "exercise_mapping.json";

fn path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|p| p.join(FILE))
}

pub fn load() {
    if let Some(p) = path() {
        if !p.exists() {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&p, include_str!("../data/default_exercise_mapping.json"));
        }
        if let Ok(data) = std::fs::read_to_string(&p) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, MuscleMapping>>(&data) {
                *MAPPINGS.lock().unwrap() = map;
            }
        }
    }
}

pub fn save() {
    if let Some(p) = path() {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&*MAPPINGS.lock().unwrap()) {
            let _ = std::fs::write(p, data);
        }
    }
}

pub fn get(ex: &str) -> Option<MuscleMapping> {
    MAPPINGS.lock().unwrap().get(ex).cloned()
}

pub fn set(ex: String, map: MuscleMapping) {
    MAPPINGS.lock().unwrap().insert(ex, map);
}

pub fn remove(ex: &str) {
    MAPPINGS.lock().unwrap().remove(ex);
}

pub fn all() -> HashMap<String, MuscleMapping> {
    MAPPINGS.lock().unwrap().clone()
}

pub fn export_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let map = all();
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data =
        serde_json::to_string_pretty(&map).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(path, data)
}

pub fn import_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let data = std::fs::read_to_string(path)?;
    let map: HashMap<String, MuscleMapping> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    *MAPPINGS.lock().unwrap() = map;
    Ok(())
}

pub fn merge_files<P: AsRef<Path>>(paths: &[P]) -> io::Result<()> {
    let mut map: HashMap<String, MuscleMapping> = all();
    let mut files: Vec<&Path> = paths.iter().map(|p| p.as_ref()).collect();
    files.sort_by(|a, b| {
        let ma = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mb = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        mb.cmp(&ma)
    });
    for p in files {
        let data = std::fs::read_to_string(p)?;
        let part: HashMap<String, MuscleMapping> =
            serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        for (k, v) in part {
            map.insert(k, v);
        }
    }
    map = map.into_iter().collect();
    *MAPPINGS.lock().unwrap() = map;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn merge_file_with_missing_fields_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        // Only provide the primary field to ensure defaults are filled in for the rest.
        writeln!(
            file,
            "{}",
            serde_json::json!({"Custom Exercise": {"primary": "Chest"}}).to_string()
        )
        .unwrap();

        merge_files(&[file.path()]).expect("merge should succeed");

        let mapping = get("Custom Exercise").expect("mapping should exist");
        assert_eq!(mapping.primary, "Chest");
        assert!(mapping.secondary.is_empty());
        assert!(mapping.category.is_empty());

        remove("Custom Exercise");
    }
}
