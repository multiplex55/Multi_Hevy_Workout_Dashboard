// Module for analyzing workout data
use crate::WorkoutEntry;
use crate::plotting::OneRmFormula;
use chrono::NaiveDate;
use std::collections::HashMap;

/// Summary statistics about a workout log.
#[derive(Debug, Default, PartialEq)]
pub struct BasicStats {
    pub total_workouts: usize,
    pub avg_sets_per_workout: f32,
    pub avg_reps_per_set: f32,
    pub avg_days_between: f32,
    pub most_common_exercise: Option<String>,
}

/// Aggregated statistics for a single exercise.
#[derive(Debug, Default, PartialEq)]
pub struct ExerciseStats {
    pub total_sets: usize,
    pub total_reps: u32,
    pub total_volume: f32,
    pub best_est_1rm: Option<f32>,
}

/// Aggregate per-exercise statistics from a slice of workout entries.
///
/// The data can be limited to an optional date range. Invalid dates are
/// skipped. For each exercise all sets are combined and the best estimated
/// one-rep max is calculated using the provided [`OneRmFormula`].
pub fn aggregate_exercise_stats(
    entries: &[WorkoutEntry],
    formula: OneRmFormula,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> HashMap<String, ExerciseStats> {
    let mut map: HashMap<String, ExerciseStats> = HashMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let stats = map
                    .entry(e.exercise.clone())
                    .or_insert_with(ExerciseStats::default);
                stats.total_sets += 1;
                stats.total_reps += e.reps;
                stats.total_volume += e.weight * e.reps as f32;

                let est = match formula {
                    OneRmFormula::Epley => e.weight * (1.0 + e.reps as f32 / 30.0),
                    OneRmFormula::Brzycki => {
                        if e.reps >= 37 {
                            continue;
                        }
                        e.weight * 36.0 / (37.0 - e.reps as f32)
                    }
                };
                stats.best_est_1rm = match stats.best_est_1rm {
                    Some(current) if current >= est => Some(current),
                    _ => Some(est),
                };
            }
        }
    }

    map
}

fn parse_date(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// Format a user facing message after successfully loading a CSV file.
///
/// The returned string includes the number of parsed entries and the file name
/// and can be used in status bars or log output.
pub fn format_load_message(entries: usize, filename: &str) -> String {
    format!("Loaded {} entries from {}", entries, filename)
}

/// Compute overall statistics for the loaded workout entries.
///
/// Only entries within the optional `start` and `end` date range are included.
/// If no valid workout dates are found an empty [`BasicStats`] is returned.
pub fn compute_stats(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> BasicStats {
    if entries.is_empty() {
        return BasicStats::default();
    }

    log::info!("Computing statistics for {} entries", entries.len());

    // Map date -> sets count within range
    let mut sets_per_day: HashMap<NaiveDate, usize> = HashMap::new();
    let mut total_reps = 0u32;
    let mut exercise_counts: HashMap<&str, usize> = HashMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                *sets_per_day.entry(d).or_insert(0) += 1;
                total_reps += e.reps;
                *exercise_counts.entry(e.exercise.as_str()).or_insert(0) += 1;
            }
        }
    }

    let total_workouts = sets_per_day.len();
    let total_sets: usize = sets_per_day.values().sum();

    if total_workouts == 0 {
        log::warn!("No valid workout dates found");
        return BasicStats::default();
    }

    let avg_sets_per_workout = total_sets as f32 / total_workouts as f32;
    let avg_reps_per_set = total_reps as f32 / total_sets as f32;

    // Days between workouts
    let mut dates: Vec<NaiveDate> = sets_per_day.keys().cloned().collect();
    dates.sort();
    let mut total_gap_days = 0i64;
    for w in dates.windows(2) {
        if let [a, b] = w {
            total_gap_days += (*b - *a).num_days();
        }
    }
    let avg_days_between = if dates.len() > 1 {
        total_gap_days as f32 / (dates.len() as f32 - 1.0)
    } else {
        0.0
    };

    let most_common_exercise = exercise_counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(ex, _)| ex.to_string());

    BasicStats {
        total_workouts,
        avg_sets_per_workout,
        avg_reps_per_set,
        avg_days_between,
        most_common_exercise,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<WorkoutEntry> {
        vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Squat".into(),
                weight: 100.0,
                reps: 5,
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 80.0,
                reps: 5,
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Squat".into(),
                weight: 105.0,
                reps: 5,
            },
            WorkoutEntry {
                date: "2024-01-05".into(),
                exercise: "Deadlift".into(),
                weight: 120.0,
                reps: 5,
            },
        ]
    }

    fn invalid_date_entries() -> Vec<WorkoutEntry> {
        vec![
            WorkoutEntry {
                date: "not-a-date".into(),
                exercise: "Squat".into(),
                weight: 100.0,
                reps: 5,
            },
            WorkoutEntry {
                date: "2024-13-01".into(), // invalid month
                exercise: "Bench".into(),
                weight: 80.0,
                reps: 5,
            },
        ]
    }

    #[test]
    fn test_compute_stats() {
        let entries = sample_entries();
        let stats = compute_stats(&entries, None, None);
        assert_eq!(stats.total_workouts, 3);
        // total sets = 4, workouts = 3 -> avg 1.333...
        assert!((stats.avg_sets_per_workout - 4f32 / 3f32).abs() < 1e-6);
        assert!((stats.avg_reps_per_set - 5.0).abs() < 1e-6);
        assert!((stats.avg_days_between - 2.0).abs() < 1e-6); // (2 + 2)/2
        assert_eq!(stats.most_common_exercise.as_deref(), Some("Squat"));
    }

    #[test]
    fn test_invalid_dates_safe_stats() {
        let entries = invalid_date_entries();
        let stats = compute_stats(&entries, None, None);
        assert_eq!(stats, BasicStats::default());
    }

    #[test]
    fn test_format_load_message() {
        let msg = format_load_message(10, "workouts.csv");
        assert_eq!(msg, "Loaded 10 entries from workouts.csv");
    }

    #[test]
    fn test_aggregate_exercise_stats() {
        let entries = sample_entries();
        let map = aggregate_exercise_stats(&entries, OneRmFormula::Epley, None, None);

        let squat = map.get("Squat").unwrap();
        assert_eq!(squat.total_sets, 2);
        assert_eq!(squat.total_reps, 10);
        assert!((squat.total_volume - 1025.0).abs() < 1e-6);
        assert!((squat.best_est_1rm.unwrap() - 122.5).abs() < 1e-3);

        let bench = map.get("Bench").unwrap();
        assert_eq!(bench.total_sets, 1);
        assert_eq!(bench.total_reps, 5);
        assert!((bench.total_volume - 400.0).abs() < 1e-6);
        assert!((bench.best_est_1rm.unwrap() - 93.3333).abs() < 1e-3);

        let deadlift = map.get("Deadlift").unwrap();
        assert_eq!(deadlift.total_sets, 1);
        assert_eq!(deadlift.total_reps, 5);
        assert!((deadlift.total_volume - 600.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_stats_with_range() {
        let entries = sample_entries();
        let start = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").ok();
        let stats = compute_stats(&entries, start, None);
        assert_eq!(stats.total_workouts, 2);
    }
}
