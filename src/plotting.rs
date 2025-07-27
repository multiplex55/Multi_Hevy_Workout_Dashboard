use chrono::{Datelike, NaiveDate};
use egui_plot::{Bar, BarChart, Line, PlotPoints};

use crate::body_parts::body_part_for;
use crate::{WeightUnit, WorkoutEntry};
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

/// Methods available for smoothing plot data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmoothingMethod {
    /// Simple moving average using a fixed window size.
    SimpleMA,
    /// Exponential moving average controlled by an alpha value.
    EMA,
}

/// How to aggregate training volume over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeAggregation {
    /// Do not aggregate, keep daily values.
    Daily,
    /// Group data by ISO week.
    Weekly,
    /// Group data by calendar month.
    Monthly,
}

impl Default for VolumeAggregation {
    fn default() -> Self {
        VolumeAggregation::Weekly
    }
}

/// Result of generating a plot line with an optional marker for the maximum value.
pub struct LineWithMarker {
    pub line: Line,
    pub points: Vec<[f64; 2]>,
    pub max_point: Option<[f64; 2]>,
    pub record_points: Vec<[f64; 2]>,
}

/// Generate a line plot of weight over time for one or more exercises.
///
/// Only entries for the listed `exercises` within the optional date range are
/// used. Invalid dates are ignored. A separate line is returned for each
/// exercise.
pub fn weight_over_time_line(
    entries: &[WorkoutEntry],
    exercises: &[String],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
    unit: WeightUnit,
    ma_window: Option<usize>,
    method: SmoothingMethod,
) -> Vec<LineWithMarker> {
    let mut lines = Vec::new();
    for exercise in exercises {
        let mut points = Vec::new();
        let mut record_points = Vec::new();
        let mut idx = 0usize;
        let mut max_weight = f64::NEG_INFINITY;
        let mut record_max = f64::NEG_INFINITY;
        let mut max_point = None;

        let mut ex_entries: Vec<&WorkoutEntry> =
            entries.iter().filter(|e| e.exercise == *exercise).collect();
        ex_entries.sort_by(|a, b| a.date.cmp(&b.date));
        for e in ex_entries {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    let x = match x_axis {
                        XAxis::Date => d.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    let f = unit.factor() as f64;
                    let weight = e.weight as f64 * f;
                    let y = match y_axis {
                        YAxis::Weight => weight,
                        YAxis::Volume => weight * e.reps as f64,
                    };
                    if e.weight as f64 * f > max_weight {
                        max_weight = e.weight as f64 * f;
                        max_point = Some([x, y]);
                    }
                    if weight > record_max {
                        record_max = weight;
                        if y_axis == YAxis::Weight {
                            record_points.push([x, y]);
                        }
                    }
                    points.push([x, y]);
                    idx += 1;
                }
            }
        }
        lines.push(LineWithMarker {
            line: Line::new(PlotPoints::from(points.clone())).name(exercise),
            points: points.clone(),
            max_point,
            record_points,
        });
        if let Some(w) = ma_window.filter(|w| *w > 1) {
            if points.len() > 1 {
                let smooth_points = match method {
                    SmoothingMethod::SimpleMA => moving_average_points(&points, w),
                    SmoothingMethod::EMA => {
                        let alpha = 2.0 / (w as f64 + 1.0);
                        ema_points(&points, alpha)
                    }
                };
                lines.push(LineWithMarker {
                    line: Line::new(PlotPoints::from(smooth_points.clone()))
                        .name(format!("{exercise} MA")),
                    points: smooth_points,
                    max_point: None,
                    record_points: Vec::new(),
                });
            }
        }
    }
    lines
}

/// Generate a line plot of the estimated one-rep max over time for a given
/// exercise.
///
/// The estimation is performed for each set using the supplied
/// [`OneRmFormula`]. Only sets for `exercise` within the optional date range are
/// included.
pub fn estimated_1rm_line(
    entries: &[WorkoutEntry],
    exercises: &[String],
    formula: OneRmFormula,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    unit: WeightUnit,
    ma_window: Option<usize>,
    method: SmoothingMethod,
) -> Vec<LineWithMarker> {
    let mut lines = Vec::new();
    for exercise in exercises {
        let mut points = Vec::new();
        let mut record_points = Vec::new();
        let mut idx = 0usize;
        let mut max_est = f64::NEG_INFINITY;
        let mut record_max = f64::NEG_INFINITY;
        let mut max_point = None;

        let mut ex_entries: Vec<&WorkoutEntry> =
            entries.iter().filter(|e| e.exercise == *exercise).collect();
        ex_entries.sort_by(|a, b| a.date.cmp(&b.date));
        for e in ex_entries {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    let f = unit.factor() as f64;
                    let est = match formula {
                        OneRmFormula::Epley => e.weight as f64 * f * (1.0 + e.reps as f64 / 30.0),
                        OneRmFormula::Brzycki => {
                            if e.reps >= 37 {
                                continue;
                            }
                            e.weight as f64 * f * 36.0 / (37.0 - e.reps as f64)
                        }
                    };
                    let x = match x_axis {
                        XAxis::Date => d.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    if est > max_est {
                        max_est = est;
                        max_point = Some([x, est]);
                    }
                    if est > record_max {
                        record_max = est;
                        record_points.push([x, est]);
                    }
                    points.push([x, est]);
                    idx += 1;
                }
            }
        }
        lines.push(LineWithMarker {
            line: Line::new(PlotPoints::from(points.clone())).name(exercise),
            points: points.clone(),
            max_point,
            record_points,
        });
        if let Some(w) = ma_window.filter(|w| *w > 1) {
            if points.len() > 1 {
                let smooth_points = match method {
                    SmoothingMethod::SimpleMA => moving_average_points(&points, w),
                    SmoothingMethod::EMA => {
                        let alpha = 2.0 / (w as f64 + 1.0);
                        ema_points(&points, alpha)
                    }
                };
                lines.push(LineWithMarker {
                    line: Line::new(PlotPoints::from(smooth_points.clone()))
                        .name(format!("{exercise} MA")),
                    points: smooth_points,
                    max_point: None,
                    record_points: Vec::new(),
                });
            }
        }
    }
    lines
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
    unit: WeightUnit,
) -> Vec<[f64; 2]> {
    let mut map: std::collections::BTreeMap<NaiveDate, (f64, f64)> =
        std::collections::BTreeMap::new();
    for e in entries {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let f = unit.factor() as f64;
                let entry = map.entry(d).or_insert((0.0, 0.0));
                entry.0 += e.weight as f64 * f * e.reps as f64; // volume
                entry.1 += e.weight as f64 * f; // total weight
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

/// Aggregate training volume by ISO week or calendar month.
pub fn aggregated_volume_points(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
    unit: WeightUnit,
    agg: VolumeAggregation,
) -> Vec<[f64; 2]> {
    use std::collections::BTreeMap;
    match agg {
        VolumeAggregation::Daily => {
            training_volume_points(entries, start, end, x_axis, y_axis, unit)
        }
        VolumeAggregation::Weekly => {
            let mut map: BTreeMap<(i32, u32), (f64, f64)> = BTreeMap::new();
            for e in entries {
                if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                    if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                        let f = unit.factor() as f64;
                        let key = (d.iso_week().year(), d.iso_week().week());
                        let entry = map.entry(key).or_insert((0.0, 0.0));
                        entry.0 += e.weight as f64 * f * e.reps as f64;
                        entry.1 += e.weight as f64 * f;
                    }
                }
            }
            let mut points = Vec::new();
            let mut idx = 0usize;
            for ((year, week), (vol, weight)) in map {
                if let Some(date) = NaiveDate::from_isoywd_opt(year, week, chrono::Weekday::Mon) {
                    let x = match x_axis {
                        XAxis::Date => date.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    let y = match y_axis {
                        YAxis::Volume => vol,
                        YAxis::Weight => weight,
                    };
                    points.push([x, y]);
                    idx += 1;
                }
            }
            points
        }
        VolumeAggregation::Monthly => {
            let mut map: BTreeMap<(i32, u32), (f64, f64)> = BTreeMap::new();
            for e in entries {
                if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                    if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                        let f = unit.factor() as f64;
                        let key = (d.year(), d.month());
                        let entry = map.entry(key).or_insert((0.0, 0.0));
                        entry.0 += e.weight as f64 * f * e.reps as f64;
                        entry.1 += e.weight as f64 * f;
                    }
                }
            }
            let mut points = Vec::new();
            let mut idx = 0usize;
            for ((year, month), (vol, weight)) in map {
                if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
                    let x = match x_axis {
                        XAxis::Date => date.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    let y = match y_axis {
                        YAxis::Volume => vol,
                        YAxis::Weight => weight,
                    };
                    points.push([x, y]);
                    idx += 1;
                }
            }
            points
        }
    }
}

/// Calculate a simple moving average of the y-values in `points`.
fn moving_average_points(points: &[[f64; 2]], window: usize) -> Vec<[f64; 2]> {
    if window == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(points.len());
    let mut sum = 0.0;
    for i in 0..points.len() {
        sum += points[i][1];
        if i >= window {
            sum -= points[i - window][1];
        }
        let count = window.min(i + 1) as f64;
        out.push([points[i][0], sum / count]);
    }
    out
}

/// Calculate an exponential moving average of the y-values in `points`.
fn ema_points(points: &[[f64; 2]], alpha: f64) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(points.len());
    let mut ema = points[0][1];
    out.push([points[0][0], ema]);
    for i in 1..points.len() {
        ema = alpha * points[i][1] + (1.0 - alpha) * ema;
        out.push([points[i][0], ema]);
    }
    out
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
    unit: WeightUnit,
    ma_window: Option<usize>,
    method: SmoothingMethod,
) -> Vec<Line> {
    let points = training_volume_points(entries, start, end, x_axis, y_axis, unit);
    let mut lines = Vec::new();
    lines.push(Line::new(PlotPoints::from(points.clone())).name("Volume"));
    if let Some(w) = ma_window.filter(|w| *w > 1) {
        if points.len() > 1 {
            let smooth = match method {
                SmoothingMethod::SimpleMA => moving_average_points(&points, w),
                SmoothingMethod::EMA => {
                    let alpha = 2.0 / (w as f64 + 1.0);
                    ema_points(&points, alpha)
                }
            };
            lines.push(Line::new(PlotPoints::from(smooth)).name(format!("Volume MA")));
        }
    }
    lines
}

/// Create a line plot of training volume per primary body part.
///
/// Each body part is plotted separately. Entries outside the optional date
/// range or without a known body part are skipped.
pub fn body_part_volume_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    unit: WeightUnit,
    ma_window: Option<usize>,
) -> Vec<Line> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, BTreeMap<NaiveDate, f64>> = BTreeMap::new();
    for e in entries {
        if let (Some(part), Ok(d)) = (
            body_part_for(&e.exercise),
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d"),
        ) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let f = unit.factor() as f64;
                *map.entry(part.to_string())
                    .or_default()
                    .entry(d)
                    .or_insert(0.0) += e.weight as f64 * f * e.reps as f64;
            }
        }
    }

    let mut lines = Vec::new();
    for (part, day_map) in map {
        let mut idx = 0usize;
        let mut points = Vec::new();
        for (d, vol) in day_map {
            let x = match x_axis {
                XAxis::Date => d.num_days_from_ce() as f64,
                XAxis::WorkoutIndex => {
                    let v = idx as f64;
                    idx += 1;
                    v
                }
            };
            points.push([x, vol]);
        }
        lines.push(Line::new(PlotPoints::from(points.clone())).name(part.clone()));
        if let Some(w) = ma_window.filter(|w| *w > 1) {
            if points.len() > 1 {
                let ma_pts = moving_average_points(&points, w);
                lines.push(Line::new(PlotPoints::from(ma_pts)).name(format!("{part} MA")));
            }
        }
    }
    lines
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
    use crate::RawWorkoutRow;
    use egui_plot::{PlotGeometry, PlotItem};

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
            WeightUnit::Lbs,
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
            WeightUnit::Lbs,
        );
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![[d3.num_days_from_ce() as f64, 525.0]];
        assert_eq!(points, expected);
    }

    #[test]
    fn test_aggregated_volume_points_weekly() {
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let pts = aggregated_volume_points(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            YAxis::Volume,
            WeightUnit::Lbs,
            VolumeAggregation::Weekly,
        );
        let expected = vec![[d.num_days_from_ce() as f64, 1425.0]];
        assert_eq!(pts, expected);
    }

    #[test]
    fn test_aggregated_volume_points_monthly() {
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let pts = aggregated_volume_points(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            YAxis::Volume,
            WeightUnit::Lbs,
            VolumeAggregation::Monthly,
        );
        let expected = vec![[d.num_days_from_ce() as f64, 1425.0]];
        assert_eq!(pts, expected);
    }

    #[test]
    fn test_moving_average_points() {
        let points = vec![[0.0, 1.0], [1.0, 3.0], [2.0, 5.0], [3.0, 7.0]];
        let ma = moving_average_points(&points, 2);
        let expected = vec![[0.0, 1.0], [1.0, 2.0], [2.0, 4.0], [3.0, 6.0]];
        assert_eq!(ma, expected);
    }

    #[test]
    fn test_ema_points() {
        let points = vec![[0.0, 1.0], [1.0, 3.0], [2.0, 5.0], [3.0, 7.0]];
        let ema = ema_points(&points, 0.5);
        let expected = vec![[0.0, 1.0], [1.0, 2.0], [2.0, 3.5], [3.0, 5.25]];
        for (a, b) in ema.iter().zip(expected.iter()) {
            assert!((a[1] - b[1]).abs() < 1e-6);
        }
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
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
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
        let res = weight_over_time_line(
            &sample_entries(),
            &["Squat".to_string()],
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Weight,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        assert_eq!(res.len(), 1);
        let lw = res.into_iter().next().unwrap();
        let expected = vec![[d1, 100.0], [d2, 105.0]];
        assert_eq!(line_points(lw.line), expected);
        assert_eq!(lw.max_point, Some([d2, 105.0]));
        assert_eq!(lw.record_points, expected);
    }

    #[test]
    fn test_training_volume_line_index_axis() {
        let line = training_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Volume,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
        let expected = vec![[0.0, 900.0], [1.0, 525.0]];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_training_volume_line_ema() {
        let line = training_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Volume,
            WeightUnit::Lbs,
            Some(2),
            SmoothingMethod::EMA,
        )
        .into_iter()
        .nth(1)
        .unwrap();
        // EMA smoothing of [0.0,900], [1.0,525] with alpha=2/(2+1)=0.666...
        let expected = vec![[0.0, 900.0], [1.0, 650.0]];
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
        let res_e = estimated_1rm_line(
            &sample_entries(),
            &["Squat".to_string()],
            OneRmFormula::Epley,
            None,
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        assert_eq!(res_e.len(), 1);
        let expected_e = vec![
            [d1.num_days_from_ce() as f64, 100.0 * (1.0 + 5.0 / 30.0)],
            [d3.num_days_from_ce() as f64, 105.0 * (1.0 + 5.0 / 30.0)],
        ];
        let lw_e = res_e.into_iter().next().unwrap();
        assert_eq!(line_points(lw_e.line), expected_e);
        assert_eq!(lw_e.max_point, Some(expected_e[1]));
        assert_eq!(lw_e.record_points, expected_e);

        let res_b = estimated_1rm_line(
            &sample_entries(),
            &["Squat".to_string()],
            OneRmFormula::Brzycki,
            Some(d3),
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        assert_eq!(res_b.len(), 1);
        let lw_b = res_b.into_iter().next().unwrap();
        let expected_b = vec![[d3.num_days_from_ce() as f64, 105.0 * 36.0 / 32.0]];
        assert_eq!(line_points(lw_b.line), expected_b);
        assert_eq!(lw_b.max_point, Some(expected_b[0]));
        assert_eq!(lw_b.record_points, expected_b);
    }
}
