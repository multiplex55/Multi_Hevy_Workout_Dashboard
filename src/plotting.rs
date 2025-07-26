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

/// Options for mapping data to the x-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XAxis {
    /// Use the workout date as the x value.
    Date,
    /// Use the position of the set in the filtered list.
    WorkoutIndex,
}

/// Options for mapping data to the y-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YAxis {
    /// Plot the set weight.
    Weight,
    /// Plot the calculated training volume (weight * reps).
    Volume,
}

/// Generate a line plot of weight over time for a given exercise.
///
/// Only entries for `exercise` within the optional date range are used. Invalid
/// dates are ignored.
pub fn weight_over_time_line(
    entries: &[WorkoutEntry],
    exercise: &str,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
) -> Line {
    let mut points = Vec::new();
    let mut idx = 0usize;
    for e in entries.iter().filter(|e| e.exercise == exercise) {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let x = match x_axis {
                    XAxis::Date => d.num_days_from_ce() as f64,
                    XAxis::WorkoutIndex => idx as f64,
                };
                let y = match y_axis {
                    YAxis::Weight => e.weight as f64,
                    YAxis::Volume => e.weight as f64 * e.reps as f64,
                };
                points.push([x, y]);
                idx += 1;
            }
        }
    }
    Line::new(PlotPoints::from(points)).name("Weight")
}

/// Generate a line plot of the estimated one-rep max over time for a given
/// exercise.
///
/// The estimation is performed for each set using the supplied
/// [`OneRmFormula`]. Only sets for `exercise` within the optional date range are
/// included.
pub fn estimated_1rm_line(
    entries: &[WorkoutEntry],
    exercise: &str,
    formula: OneRmFormula,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
) -> Line {
    let mut points = Vec::new();
    let mut idx = 0usize;
    for e in entries.iter().filter(|e| e.exercise == exercise) {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let est = match formula {
                    OneRmFormula::Epley => e.weight as f64 * (1.0 + e.reps as f64 / 30.0),
                    OneRmFormula::Brzycki => {
                        if e.reps >= 37 {
                            continue;
                        }
                        e.weight as f64 * 36.0 / (37.0 - e.reps as f64)
                    }
                };
                let x = match x_axis {
                    XAxis::Date => d.num_days_from_ce() as f64,
                    XAxis::WorkoutIndex => idx as f64,
                };
                points.push([x, est]);
                idx += 1;
            }
        }
    }
    Line::new(PlotPoints::from(points)).name("1RM Est")
}

/// Create a bar chart of how many sets were performed on each day.
///
/// When `exercise` is `Some`, only sets of that exercise are counted. Entries
/// outside of the optional date range or with invalid dates are skipped.
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
    x_axis: XAxis,
    y_axis: YAxis,
) -> Vec<[f64; 2]> {
    let mut map: std::collections::BTreeMap<NaiveDate, (f64, f64)> =
        std::collections::BTreeMap::new();
    for e in entries {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let entry = map.entry(d).or_insert((0.0, 0.0));
                entry.0 += e.weight as f64 * e.reps as f64; // volume
                entry.1 += e.weight as f64; // total weight
            }
        }
    }
    let mut points = Vec::new();
    let mut idx = 0usize;
    for (d, (vol, weight)) in map {
        let x = match x_axis {
            XAxis::Date => d.num_days_from_ce() as f64,
            XAxis::WorkoutIndex => idx as f64,
        };
        let y = match y_axis {
            YAxis::Volume => vol,
            YAxis::Weight => weight,
        };
        points.push([x, y]);
        idx += 1;
    }
    points
}

/// Create a line plot of total training volume per day.
///
/// Training volume is calculated as `weight * reps` for each set. Only entries
/// within the optional date range are considered.
pub fn training_volume_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
) -> Line {
    let points = training_volume_points(entries, start, end, x_axis, y_axis);
    Line::new(PlotPoints::from(points)).name("Volume")
}

/// Return a sorted list of unique exercises found in the data.
///
/// Only entries whose dates fall inside the optional range are inspected. The
/// resulting vector is sorted alphabetically.
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
    use egui_plot::{PlotGeometry, PlotItem};
    use crate::RawWorkoutRow;

    fn sample_entries() -> Vec<WorkoutEntry> {
        vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Squat".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 80.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Squat".into(),
                weight: 105.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
        ]
    }

    #[test]
    fn test_training_volume_points() {
        let points = training_volume_points(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            YAxis::Volume,
        );
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
        let points = training_volume_points(
            &sample_entries(),
            start,
            None,
            XAxis::Date,
            YAxis::Volume,
        );
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![[d3.num_days_from_ce() as f64, 525.0]];
        assert_eq!(points, expected);
    }

    fn line_points(line: Line) -> Vec<[f64; 2]> {
        if let PlotGeometry::Points(points) = line.geometry() {
            points.iter().map(|p| [p.x, p.y]).collect()
        } else {
            panic!("expected points")
        }
    }

    #[test]
    fn test_training_volume_line() {
        let line = training_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            YAxis::Volume,
        );
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![
            [d1.num_days_from_ce() as f64, 900.0],
            [d3.num_days_from_ce() as f64, 525.0],
        ];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_weight_over_time_index_axis() {
        let d1 = 0.0;
        let d2 = 1.0;
        let line = weight_over_time_line(
            &sample_entries(),
            "Squat",
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Weight,
        );
        let expected = vec![[d1, 100.0], [d2, 105.0]];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_training_volume_line_index_axis() {
        let line = training_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Volume,
        );
        let expected = vec![[0.0, 900.0], [1.0, 525.0]];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_unique_exercises() {
        let ex = unique_exercises(&sample_entries(), None, None);
        assert_eq!(ex, vec!["Bench".to_string(), "Squat".to_string()]);
    }

    #[test]
    fn test_unique_exercises_range() {
        let start = NaiveDate::parse_from_str("2024-01-02", "%Y-%m-%d").ok();
        let ex = unique_exercises(&sample_entries(), start, None);
        assert_eq!(ex, vec!["Squat".to_string()]);
    }

    #[test]
    fn test_estimated_1rm_line_formulas_and_range() {
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let line_e = estimated_1rm_line(
            &sample_entries(),
            "Squat",
            OneRmFormula::Epley,
            None,
            None,
            XAxis::Date,
        );
        let expected_e = vec![
            [d1.num_days_from_ce() as f64, 100.0 * (1.0 + 5.0 / 30.0)],
            [d3.num_days_from_ce() as f64, 105.0 * (1.0 + 5.0 / 30.0)],
        ];
        assert_eq!(line_points(line_e), expected_e);

        let line_b = estimated_1rm_line(
            &sample_entries(),
            "Squat",
            OneRmFormula::Brzycki,
            Some(d3),
            None,
            XAxis::Date,
        );
        let expected_b = vec![[d3.num_days_from_ce() as f64, 105.0 * 36.0 / 32.0]];
        assert_eq!(line_points(line_b), expected_b);
    }
}
