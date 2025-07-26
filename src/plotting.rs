use chrono::{Datelike, NaiveDate};
use egui_plot::{Bar, BarChart, Line, PlotPoints};

use crate::WorkoutEntry;

/// Generate a line plot of weight over time for a given exercise.
pub fn weight_over_time_line(entries: &[WorkoutEntry], exercise: &str) -> Line {
    let points: Vec<[f64; 2]> = entries
        .iter()
        .filter(|e| e.exercise == exercise)
        .filter_map(|e| {
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                .ok()
                .map(|d| [d.num_days_from_ce() as f64, e.weight as f64])
        })
        .collect();
    Line::new(PlotPoints::from(points)).name("Weight")
}

/// Generate a line plot of estimated 1RM over time for a given exercise.
/// Uses the Epley formula: weight * (1 + reps / 30).
pub fn estimated_1rm_line(entries: &[WorkoutEntry], exercise: &str) -> Line {
    let points: Vec<[f64; 2]> = entries
        .iter()
        .filter(|e| e.exercise == exercise)
        .filter_map(|e| {
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                .ok()
                .map(|d| {
                    let est = e.weight as f64 * (1.0 + e.reps as f64 / 30.0);
                    [d.num_days_from_ce() as f64, est]
                })
        })
        .collect();
    Line::new(PlotPoints::from(points)).name("1RM Est")
}

/// Create a bar chart of sets per day for an optional exercise.
pub fn sets_per_day_bar(entries: &[WorkoutEntry], exercise: Option<&str>) -> BarChart {
    let mut map: std::collections::BTreeMap<NaiveDate, usize> = std::collections::BTreeMap::new();
    for e in entries {
        if exercise.map(|ex| ex == e.exercise).unwrap_or(true) {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                *map.entry(d).or_insert(0) += 1;
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

/// Return a sorted list of unique exercises found in the data.
pub fn unique_exercises(entries: &[WorkoutEntry]) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for e in entries {
        set.insert(e.exercise.clone());
    }
    set.into_iter().collect()
}
