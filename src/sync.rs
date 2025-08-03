use crate::{RawWorkoutRow, WorkoutEntry};
use serde_json::Value;

const HEVY_URL: &str = "https://api.hevyapp.com/v1/workouts";

/// Fetch the latest workouts from the Hevy API using the provided API key.
///
/// The function performs a simple HTTP GET request to the public Hevy
/// endpoint and attempts to map the returned JSON into the existing
/// `WorkoutEntry` structure. Only a subset of fields is extracted so the
/// function remains resilient to API changes. Any missing data is skipped.
pub fn fetch_latest_workouts(
    api_key: &str,
) -> Result<Vec<WorkoutEntry>, Box<dyn std::error::Error>> {
    let resp = ureq::get(HEVY_URL)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Accept", "application/json")
        .call()?
        .into_string()?;
    let json: Value = serde_json::from_str(&resp)?;
    let mut entries = Vec::new();
    if let Some(workouts) = json.as_array() {
        for w in workouts {
            let start_time = w.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
            let date = start_time.split('T').next().unwrap_or("").to_string();
            if let Some(exercises) = w.get("exercises").and_then(|v| v.as_array()) {
                for ex in exercises {
                    let name = ex.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    if let Some(sets) = ex.get("sets").and_then(|v| v.as_array()) {
                        for set in sets {
                            let weight = set
                                .get("weight")
                                .or_else(|| set.get("weight_kg"))
                                .or_else(|| set.get("weight_lb"))
                                .and_then(|v| v.as_f64());
                            let reps = set.get("reps").and_then(|v| v.as_u64());
                            if let (Some(weight), Some(reps)) = (weight, reps) {
                                let mut raw = RawWorkoutRow::default();
                                raw.start_time = start_time.to_string();
                                raw.exercise_title = name.to_string();
                                raw.weight_kg = Some(weight as f32);
                                raw.reps = Some(reps as u32);
                                let entry = WorkoutEntry {
                                    date: date.clone(),
                                    exercise: name.to_string(),
                                    weight: Some(weight as f32 * 2.20462),
                                    reps: Some(reps as u32),
                                    raw,
                                };
                                entries.push(entry);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(entries)
}
