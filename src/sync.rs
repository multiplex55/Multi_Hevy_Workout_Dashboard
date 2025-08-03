use crate::{RawWorkoutRow, WorkoutEntry};
use serde_json::Value;

const HEVY_URL: &str = "https://api.hevyapp.com/v1/workouts";

/// Determine the API key to use for Hevy requests.
///
/// If the `HEVY_API_KEY` environment variable is set, its value takes
/// precedence over any key provided in the application settings.
pub fn resolve_api_key(settings_key: Option<&str>) -> Option<String> {
    std::env::var("HEVY_API_KEY").ok().or_else(|| settings_key.map(|s| s.to_string()))
}

#[derive(Debug)]
pub enum SyncError {
    Unauthorized(String),
    Forbidden(String),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Unauthorized(body) => write!(f, "Unauthorized: {body}"),
            SyncError::Forbidden(body) => write!(f, "Forbidden: {body}"),
            SyncError::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SyncError::Unauthorized(_) | SyncError::Forbidden(_) => None,
            SyncError::Other(e) => Some(&**e),
        }
    }
}

fn fetch_latest_workouts_with_url(
    url: &str,
    api_key: &str,
    after: Option<&str>,
) -> Result<Vec<WorkoutEntry>, SyncError> {
    let mut req = ureq::get(url);
    if let Some(ts) = after {
        req = req.query("after", ts);
    }
    let response = req
        .set("X-API-Key", api_key)
        .set("Accept", "application/json")
        .call();
    let resp = match response {
        Ok(r) => r.into_string().map_err(|e| SyncError::Other(Box::new(e)))?,
        Err(ureq::Error::Status(401, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(SyncError::Unauthorized(body));
        }
        Err(ureq::Error::Status(403, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(SyncError::Forbidden(body));
        }
        Err(e) => return Err(SyncError::Other(Box::new(e))),
    };
    let json: Value = serde_json::from_str(&resp).map_err(|e| SyncError::Other(Box::new(e)))?;
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

/// Fetch the latest workouts from the Hevy API using the provided API key.
///
/// The function performs a simple HTTP GET request to the public Hevy
/// endpoint and attempts to map the returned JSON into the existing
/// `WorkoutEntry` structure. Only a subset of fields is extracted so the
/// function remains resilient to API changes. Any missing data is skipped.
pub fn fetch_latest_workouts(
    api_key: &str,
    after: Option<&str>,
) -> Result<Vec<WorkoutEntry>, SyncError> {
    log::info!("Fetching latest workouts using API key: {api_key}");
    fetch_latest_workouts_with_url(HEVY_URL, api_key, after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn maps_403_to_forbidden() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/v1/workouts");
            then.status(403).body("forbidden body");
        });

        let err =
            fetch_latest_workouts_with_url(&server.url("/v1/workouts"), "key", None).unwrap_err();
        match err {
            SyncError::Forbidden(body) => assert_eq!(body, "forbidden body"),
            e => panic!("unexpected error: {e:?}"),
        }

        m.assert();
    }

    #[test]
    fn maps_401_to_unauthorized() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/v1/workouts");
            then.status(401).body("unauthorized body");
        });

        let err =
            fetch_latest_workouts_with_url(&server.url("/v1/workouts"), "key", None).unwrap_err();
        match err {
            SyncError::Unauthorized(body) => assert_eq!(body, "unauthorized body"),
            e => panic!("unexpected error: {e:?}"),
        }

        m.assert();
    }

    #[test]
    fn env_var_overrides_settings_key() {
        unsafe {
            std::env::set_var("HEVY_API_KEY", "forced");
        }

        let key = resolve_api_key(Some("settings_key"));
        assert_eq!(key.as_deref(), Some("forced"));

        // Ensure the selected key is sent in the request
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/workouts")
                .header("X-API-Key", "forced");
            then.status(200).body("[]");
        });

        fetch_latest_workouts_with_url(&server.url("/v1/workouts"), key.as_deref().unwrap(), None)
            .unwrap();

        m.assert();

        unsafe {
            std::env::remove_var("HEVY_API_KEY");
        }
    }
}
