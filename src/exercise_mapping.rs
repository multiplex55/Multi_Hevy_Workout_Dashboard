use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Mutex;

use dirs_next as dirs;

use crate::body_parts;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

pub fn all_with_defaults() -> HashMap<String, MuscleMapping> {
    let guard = MAPPINGS.lock().unwrap();
    let mut map = HashMap::new();
    for ex in body_parts::EXERCISES.keys() {
        map.insert(
            (*ex).to_string(),
            guard.get(*ex).cloned().unwrap_or_default(),
        );
    }
    map
}

pub fn export_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let map = all_with_defaults();
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data =
        serde_json::to_string_pretty(&map).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(path, data)
}

pub fn import_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let data = std::fs::read_to_string(path)?;
    let mut map: HashMap<String, MuscleMapping> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    for ex in body_parts::EXERCISES.keys() {
        map.entry((*ex).to_string()).or_default();
    }
    *MAPPINGS.lock().unwrap() = map;
    Ok(())
}
