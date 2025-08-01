// Module for analyzing workout data
use crate::WorkoutEntry;
use crate::body_parts::body_part_for;
use crate::plotting::OneRmFormula;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// Summary statistics about a workout log.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BasicStats {
    pub total_workouts: usize,
    pub avg_sets_per_workout: f32,
    pub avg_reps_per_set: f32,
    pub avg_days_between: f32,
    pub most_common_exercise: Option<String>,
}

/// Aggregated statistics for a single exercise.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExerciseStats {
    pub total_sets: usize,
    pub total_reps: u32,
    pub total_volume: f32,
    pub best_est_1rm: Option<f32>,
    pub max_weight: Option<f32>,
    /// Slope of the average set weight over time.
    pub weight_trend: Option<f32>,
    /// Slope of the training volume over time.
    pub volume_trend: Option<f32>,
}

/// Personal record values for a single exercise.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExerciseRecord {
    /// Highest weight lifted in any set.
    pub max_weight: Option<f32>,
    /// Maximum training volume (weight * reps) for a single set.
    pub max_volume: Option<f32>,
    /// Best estimated one-rep max across all sets.
    pub best_est_1rm: Option<f32>,
    /// Best weight achieved for each rep count.
    pub rep_prs: HashMap<u32, f32>,
}

/// Weekly aggregate totals for sets and training volume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeeklySummary {
    /// ISO week year.
    pub year: i32,
    /// ISO week number within the year.
    pub week: u32,
    /// Total number of sets performed in the week.
    pub total_sets: usize,
    /// Total training volume (weight * reps) for the week in lbs.
    pub total_volume: f32,
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
    let mut trend_data: HashMap<String, Vec<(f32, f32, f32)>> = HashMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let stats = map
                    .entry(e.exercise.clone())
                    .or_insert_with(ExerciseStats::default);
                stats.total_sets += 1;
                stats.total_reps += e.reps;
                stats.total_volume += e.weight * e.reps as f32;

                stats.max_weight = match stats.max_weight {
                    Some(current) if current >= e.weight => Some(current),
                    _ => Some(e.weight),
                };

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

                // Scale the time axis so slope represents change per month
                let t = d.num_days_from_ce() as f32 / 30.0;
                trend_data.entry(e.exercise.clone()).or_default().push((
                    t,
                    e.weight,
                    e.weight * e.reps as f32,
                ));
            }
        }
    }

    for (ex, data) in trend_data {
        let mut weight_pts = Vec::new();
        let mut volume_pts = Vec::new();
        for (t, w, v) in data {
            weight_pts.push((t, w));
            volume_pts.push((t, v));
        }
        if let Some(stats) = map.get_mut(&ex) {
            stats.weight_trend = Some(slope(&weight_pts));
            stats.volume_trend = Some(slope(&volume_pts));
        }
    }

    map
}

/// Count how many sets target each primary body part.
///
/// Entries outside the optional date range are ignored. Exercises are mapped
/// to a body part via [`body_part_for`]. The resulting map uses the body part
/// name as the key with the number of sets as the value.
pub fn aggregate_sets_by_body_part(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = HashMap::new();
    for e in entries {
        if let (Some(bp), Some(d)) = (body_part_for(&e.exercise), parse_date(&e.date)) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                *map.entry(bp).or_insert(0) += 1;
            }
        }
    }
    map
}

/// Determine personal records for each exercise.
///
/// Returns the maximum weight, volume and estimated one-rep max found in the
/// provided entries for every exercise. Only entries within the optional date
/// range are considered.
pub fn personal_records(
    entries: &[WorkoutEntry],
    formula: OneRmFormula,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> HashMap<String, ExerciseRecord> {
    let mut map: HashMap<String, ExerciseRecord> = HashMap::new();
    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let rec = map.entry(e.exercise.clone()).or_default();
                rec.max_weight = match rec.max_weight {
                    Some(w) if w >= e.weight => Some(w),
                    _ => Some(e.weight),
                };
                let vol = e.weight * e.reps as f32;
                rec.max_volume = match rec.max_volume {
                    Some(v) if v >= vol => Some(v),
                    _ => Some(vol),
                };
                rec.rep_prs
                    .entry(e.reps)
                    .and_modify(|w| {
                        if e.weight > *w {
                            *w = e.weight;
                        }
                    })
                    .or_insert(e.weight);
                let est = match formula {
                    OneRmFormula::Epley => e.weight * (1.0 + e.reps as f32 / 30.0),
                    OneRmFormula::Brzycki => {
                        if e.reps >= 37 {
                            continue;
                        }
                        e.weight * 36.0 / (37.0 - e.reps as f32)
                    }
                };
                rec.best_est_1rm = match rec.best_est_1rm {
                    Some(b) if b >= est => Some(b),
                    _ => Some(est),
                };
            }
        }
    }
    map
}

/// Aggregate total sets and volume for each ISO week.
///
/// The returned vector is sorted by `(year, week)`.
pub fn aggregate_weekly_summary(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Vec<WeeklySummary> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<(i32, u32), WeeklySummary> = BTreeMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let iso = d.iso_week();
                let key = (iso.year(), iso.week());
                let entry = map.entry(key).or_insert(WeeklySummary {
                    year: iso.year(),
                    week: iso.week(),
                    total_sets: 0,
                    total_volume: 0.0,
                });
                entry.total_sets += 1;
                entry.total_volume += e.weight * e.reps as f32;
            }
        }
    }

    map.into_iter().map(|(_, v)| v).collect()
}

fn parse_date(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

fn slope(points: &[(f32, f32)]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    let n = points.len() as f32;
    let mean_x: f32 = points.iter().map(|(x, _)| *x).sum::<f32>() / n;
    let mean_y: f32 = points.iter().map(|(_, y)| *y).sum::<f32>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in points {
        num += (*x - mean_x) * (*y - mean_y);
        den += (*x - mean_x) * (*x - mean_x);
    }
    if den == 0.0 { 0.0 } else { num / den }
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

    // Map unique workout identifier -> sets count within range
    let mut sets_per_workout: HashMap<String, usize> = HashMap::new();
    // Track the date of each unique workout for gap calculations
    let mut workout_dates: HashMap<String, NaiveDate> = HashMap::new();
    let mut total_reps = 0u32;
    let mut exercise_counts: HashMap<&str, usize> = HashMap::new();

    for e in entries {
        if let Some(d) = parse_date(&e.date) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let id = format!(
                    "{}{}",
                    e.raw.title.as_deref().unwrap_or(""),
                    e.raw.start_time
                );
                *sets_per_workout.entry(id.clone()).or_insert(0) += 1;
                workout_dates.entry(id).or_insert(d);
                total_reps += e.reps;
                *exercise_counts.entry(e.exercise.as_str()).or_insert(0) += 1;
            }
        }
    }

    let total_workouts = sets_per_workout.len();
    let total_sets: usize = sets_per_workout.values().sum();

    if total_workouts == 0 {
        log::warn!("No valid workout dates found");
        return BasicStats::default();
    }

    let avg_sets_per_workout = total_sets as f32 / total_workouts as f32;
    let avg_reps_per_set = total_reps as f32 / total_sets as f32;

    // Days between workouts
    let mut dates: Vec<NaiveDate> = workout_dates.values().cloned().collect();
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

/// Return a sorted list of unique set types found in the data.
pub fn unique_set_types(entries: &[WorkoutEntry]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for e in entries {
        if let Some(ref t) = e.raw.set_type {
            set.insert(t.clone());
        }
    }
    set.into_iter().collect()
}

/// Return a sorted list of unique superset ids found in the data.
pub fn unique_superset_ids(entries: &[WorkoutEntry]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for e in entries {
        if let Some(ref id) = e.raw.superset_id {
            set.insert(id.clone());
        }
    }
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawWorkoutRow;

    fn sample_entries() -> Vec<WorkoutEntry> {
        vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Squat".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow {
                    title: Some("Workout 1".into()),
                    start_time: "01 Jan 2024, 10:00".into(),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 80.0,
                reps: 5,
                raw: RawWorkoutRow {
                    title: Some("Workout 1".into()),
                    start_time: "01 Jan 2024, 10:00".into(),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Squat".into(),
                weight: 105.0,
                reps: 5,
                raw: RawWorkoutRow {
                    title: Some("Workout 2".into()),
                    start_time: "03 Jan 2024, 10:00".into(),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-05".into(),
                exercise: "Deadlift".into(),
                weight: 120.0,
                reps: 5,
                raw: RawWorkoutRow {
                    title: Some("Workout 3".into()),
                    start_time: "05 Jan 2024, 10:00".into(),
                    ..RawWorkoutRow::default()
                },
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
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-13-01".into(), // invalid month
                exercise: "Bench".into(),
                weight: 80.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
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
        assert!((squat.max_weight.unwrap() - 105.0).abs() < 1e-6);
        assert!(squat.weight_trend.unwrap() > 0.0);
        assert!(squat.volume_trend.unwrap() > 0.0);

        let bench = map.get("Bench").unwrap();
        assert_eq!(bench.total_sets, 1);
        assert_eq!(bench.total_reps, 5);
        assert!((bench.total_volume - 400.0).abs() < 1e-6);
        assert!((bench.best_est_1rm.unwrap() - 93.3333).abs() < 1e-3);
        assert!((bench.max_weight.unwrap() - 80.0).abs() < 1e-6);
        assert_eq!(bench.weight_trend.unwrap(), 0.0);
        assert_eq!(bench.volume_trend.unwrap(), 0.0);

        let deadlift = map.get("Deadlift").unwrap();
        assert_eq!(deadlift.total_sets, 1);
        assert_eq!(deadlift.total_reps, 5);
        assert!((deadlift.total_volume - 600.0).abs() < 1e-6);
        assert!((deadlift.max_weight.unwrap() - 120.0).abs() < 1e-6);
        assert_eq!(deadlift.weight_trend.unwrap(), 0.0);
        assert_eq!(deadlift.volume_trend.unwrap(), 0.0);
    }

    #[test]
    fn test_compute_stats_with_range() {
        let entries = sample_entries();
        let start = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").ok();
        let stats = compute_stats(&entries, start, None);
        assert_eq!(stats.total_workouts, 2);
    }

    #[test]
    fn test_aggregate_sets_by_body_part() {
        let entries = sample_entries();
        let map = aggregate_sets_by_body_part(&entries, None, None);
        assert_eq!(map.get("Quads"), Some(&2));
        assert_eq!(map.get("Chest"), Some(&1));
        assert_eq!(map.get("Back"), Some(&1));
    }

    #[test]
    fn test_aggregate_sets_by_body_part_range() {
        let entries = sample_entries();
        let start = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").ok();
        let map = aggregate_sets_by_body_part(&entries, start, None);
        assert_eq!(map.get("Quads"), Some(&1));
        assert_eq!(map.get("Back"), Some(&1));
        assert!(map.get("Chest").is_none());
    }

    #[test]
    fn test_unique_set_types_and_superset_ids() {
        let mut entries = sample_entries();
        entries[0].raw.set_type = Some("warmup".into());
        entries[1].raw.set_type = Some("working".into());
        entries[2].raw.set_type = Some("working".into());
        entries[0].raw.superset_id = Some("A".into());
        entries[3].raw.superset_id = Some("B".into());
        let types = unique_set_types(&entries);
        assert_eq!(
            types,
            vec!["warmup", "working"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        let ids = unique_superset_ids(&entries);
        assert_eq!(
            ids,
            vec!["A", "B"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_personal_records() {
        let entries = sample_entries();
        let map = personal_records(&entries, OneRmFormula::Epley, None, None);

        let squat = map.get("Squat").unwrap();
        assert!((squat.max_weight.unwrap() - 105.0).abs() < 1e-6);
        assert!((squat.max_volume.unwrap() - 525.0).abs() < 1e-6);
        assert!((squat.best_est_1rm.unwrap() - 122.5).abs() < 1e-3);
        assert!((squat.rep_prs.get(&5).unwrap() - 105.0).abs() < 1e-6);

        let bench = map.get("Bench").unwrap();
        assert!((bench.max_weight.unwrap() - 80.0).abs() < 1e-6);
        assert!((bench.max_volume.unwrap() - 400.0).abs() < 1e-6);
        assert!((bench.best_est_1rm.unwrap() - 93.3333).abs() < 1e-3);
        assert!((bench.rep_prs.get(&5).unwrap() - 80.0).abs() < 1e-6);

        let deadlift = map.get("Deadlift").unwrap();
        assert!((deadlift.max_weight.unwrap() - 120.0).abs() < 1e-6);
        assert!((deadlift.max_volume.unwrap() - 600.0).abs() < 1e-6);
        assert!((deadlift.rep_prs.get(&5).unwrap() - 120.0).abs() < 1e-6);
    }

    #[test]
    fn test_aggregate_weekly_summary() {
        let entries = sample_entries();
        let weeks = aggregate_weekly_summary(&entries, None, None);
        assert_eq!(weeks.len(), 1);
        let w = &weeks[0];
        assert_eq!(w.year, 2024);
        assert_eq!(w.week, 1);
        assert_eq!(w.total_sets, 4);
        assert!((w.total_volume - 2025.0).abs() < 1e-6);
    }

    #[test]
    fn test_aggregate_weekly_summary_range() {
        let entries = sample_entries();
        let start = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").ok();
        let weeks = aggregate_weekly_summary(&entries, start, None);
        assert_eq!(weeks.len(), 1);
        let w = &weeks[0];
        assert_eq!(w.total_sets, 2);
        assert!((w.total_volume - 1125.0).abs() < 1e-6);
    }
}
