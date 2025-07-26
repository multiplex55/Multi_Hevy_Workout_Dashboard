use chrono::{Datelike, NaiveDate};
use egui_plot::{Bar, BarChart, Line, PlotPoints};

use crate::WorkoutEntry;
use serde::{Deserialize, Serialize};

/// Available formulas for estimating a one-rep max.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneRmFormula {
    /// Epley formula: `weight * (1 + reps / 30)`.
    Epley,
    /// Brzycki formula: `weight * 36 / (37 - reps)`.
    Brzycki,
}

/// Generate a line plot of weight over time for a given exercise.
pub fn weight_over_time_line(
    entries: &[WorkoutEntry],
    exercise: &str,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Line {
    let points: Vec<[f64; 2]> = entries
        .iter()
        .filter(|e| e.exercise == exercise)
        .filter_map(|e| {
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                .ok()
                .filter(|d| start.map_or(true, |s| *d >= s) && end.map_or(true, |e2| *d <= e2))
                .map(|d| [d.num_days_from_ce() as f64, e.weight as f64])
        })
        .collect();
    Line::new(PlotPoints::from(points)).name("Weight")
}

/// Generate a line plot of estimated 1RM over time for a given exercise.
///
/// * `entries` - All workout entries loaded from the CSV.
/// * `exercise` - Name of the exercise to plot.
/// * `formula` - The one-rep max estimation formula to use.
pub fn estimated_1rm_line(
    entries: &[WorkoutEntry],
    exercise: &str,
    formula: OneRmFormula,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Line {
    let points: Vec<[f64; 2]> = entries
        .iter()
        .filter(|e| e.exercise == exercise)
        .filter_map(|e| {
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                .ok()
                .filter(|d| start.map_or(true, |s| *d >= s) && end.map_or(true, |e2| *d <= e2))
                .and_then(|d| {
                    let est = match formula {
                        OneRmFormula::Epley => e.weight as f64 * (1.0 + e.reps as f64 / 30.0),
                        OneRmFormula::Brzycki => {
                            if e.reps >= 37 {
                                return None;
                            }
                            e.weight as f64 * 36.0 / (37.0 - e.reps as f64)
                        }
                    };
                    Some([d.num_days_from_ce() as f64, est])
                })
        })
        .collect();
    Line::new(PlotPoints::from(points)).name("1RM Est")
}

/// Create a bar chart of sets per day for an optional exercise.
pub fn sets_per_day_bar(
    entries: &[WorkoutEntry],
    exercise: Option<&str>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> BarChart {
    let mut map: std::collections::BTreeMap<NaiveDate, usize> = std::collections::BTreeMap::new();
    for e in entries {
        if exercise.map(|ex| ex == e.exercise).unwrap_or(true) {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    *map.entry(d).or_insert(0) += 1;
                }
            }
        }
    }
    let bars: Vec<Bar> = map
        .into_iter()
        .enumerate()
        .map(|(idx, (_d, count))| Bar::new(idx as f64, count as f64))
        .collect();
    BarChart::new(bars).name("Sets")
}

/// Calculate total training volume (weight * reps) per workout date.
fn training_volume_points(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Vec<[f64; 2]> {
    let mut map: std::collections::BTreeMap<NaiveDate, f64> = std::collections::BTreeMap::new();
    for e in entries {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                *map.entry(d).or_insert(0.0) += e.weight as f64 * e.reps as f64;
            }
        }
    }
    map.into_iter()
        .map(|(d, vol)| [d.num_days_from_ce() as f64, vol])
        .collect()
}

/// Create a line plot of training volume per day.
pub fn training_volume_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Line {
    let points = training_volume_points(entries, start, end);
    Line::new(PlotPoints::from(points)).name("Volume")
}

/// Return a sorted list of unique exercises found in the data.
pub fn unique_exercises(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for e in entries {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                set.insert(e.exercise.clone());
            }
        }
    }
    set.into_iter().collect()
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
        ]
    }

    #[test]
    fn test_training_volume_points() {
        let points = training_volume_points(&sample_entries(), None, None);
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![
            [d1.num_days_from_ce() as f64, 900.0],
            [d3.num_days_from_ce() as f64, 525.0],
        ];
        assert_eq!(points, expected);
    }

    #[test]
    fn test_training_volume_points_range() {
        let start = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").ok();
        let points = training_volume_points(&sample_entries(), start, None);
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![[d3.num_days_from_ce() as f64, 525.0]];
        assert_eq!(points, expected);
    }
}
