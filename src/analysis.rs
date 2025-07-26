// Module for analyzing workout data
use crate::WorkoutEntry;
use chrono::{NaiveDate};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq)]
pub struct BasicStats {
    pub total_workouts: usize,
    pub avg_sets_per_workout: f32,
    pub avg_reps_per_set: f32,
    pub avg_days_between: f32,
    pub most_common_exercise: Option<String>,
}

fn parse_date(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

pub fn compute_stats(entries: &[WorkoutEntry]) -> BasicStats {
    if entries.is_empty() {
        return BasicStats::default();
    }

    // Map date -> sets count
    let mut sets_per_day: HashMap<NaiveDate, usize> = HashMap::new();
    let mut total_reps = 0u32;
    let mut exercise_counts: HashMap<&str, usize> = HashMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            *sets_per_day.entry(d).or_insert(0) += 1;
        }
        total_reps += e.reps;
        *exercise_counts.entry(e.exercise.as_str()).or_insert(0) += 1;
    }

    let total_workouts = sets_per_day.len();
    let total_sets = entries.len();

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
            WorkoutEntry { date: "2024-01-01".into(), exercise: "Squat".into(), weight: 100.0, reps: 5 },
            WorkoutEntry { date: "2024-01-01".into(), exercise: "Bench".into(), weight: 80.0, reps: 5 },
            WorkoutEntry { date: "2024-01-03".into(), exercise: "Squat".into(), weight: 105.0, reps: 5 },
            WorkoutEntry { date: "2024-01-05".into(), exercise: "Deadlift".into(), weight: 120.0, reps: 5 },
        ]
    }

    #[test]
    fn test_compute_stats() {
        let entries = sample_entries();
        let stats = compute_stats(&entries);
        assert_eq!(stats.total_workouts, 3);
        // total sets = 4, workouts = 3 -> avg 1.333...
        assert!((stats.avg_sets_per_workout - 4f32/3f32).abs() < 1e-6);
        assert!((stats.avg_reps_per_set - 5.0).abs() < 1e-6);
        assert!((stats.avg_days_between - 2.0).abs() < 1e-6); // (2 + 2)/2
        assert_eq!(stats.most_common_exercise.as_deref(), Some("Squat"));
    }
}
