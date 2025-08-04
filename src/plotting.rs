use chrono::{Datelike, NaiveDate};
use egui::epaint::Hsva;
use egui::{Align2, Color32, FontId, Pos2, Sense, Shape, Stroke, Ui, Vec2};
use egui_plot::{Bar, BarChart, HLine, Line, PlotPoints, PlotUi, Points, VLine};

use crate::body_parts::body_part_for;
use crate::{
    WeightUnit, WorkoutEntry,
    analysis::{
        WeeklySummary, aggregate_rep_counts, aggregate_sets_by_body_part, linear_projection,
    },
    exercise_utils::normalize_exercise,
};
use serde::{Deserialize, Serialize};

/// Available formulas for estimating a one-rep max.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneRmFormula {
    /// Epley formula: `weight * (1 + reps / 30)`.
    Epley,
    /// Brzycki formula: `weight * 36 / (37 - reps)`.
    ///
    /// Undefined when `reps >= 37`.
    Brzycki,
    /// Lombardi formula: `weight * reps^0.10`.
    ///
    /// No specific rep limit.
    Lombardi,
    /// Mayhew et al. formula: `100 * weight / (52.2 + 41.9 * e^(-0.055 * reps))`.
    ///
    /// No specific rep limit.
    Mayhew,
    /// O'Conner et al. formula: `weight * (1 + reps / 40)`.
    ///
    /// No specific rep limit.
    OConner,
    /// Wathan formula: `100 * weight / (48.8 + 53.8 * e^(-0.075 * reps))`.
    ///
    /// No specific rep limit.
    Wathan,
    /// Lander formula: `weight / (1.013 - 0.0267123 * reps)`.
    ///
    /// Undefined when `reps >= 1.013 / 0.0267123` (~37.9).
    Lander,
}

impl OneRmFormula {
    /// Estimate a one-rep max for the given `weight` and `reps`.
    ///
    /// Returns `None` if the formula is undefined for the supplied inputs
    /// (e.g. Brzycki with reps >= 37).
    pub fn estimate(self, weight: f64, reps: u32) -> Option<f64> {
        let r = reps as f64;
        match self {
            OneRmFormula::Epley => Some(weight * (1.0 + r / 30.0)),
            OneRmFormula::Brzycki => {
                if reps >= 37 {
                    None
                } else {
                    Some(weight * 36.0 / (37.0 - r))
                }
            }
            OneRmFormula::Lombardi => Some(weight * r.powf(0.10)),
            OneRmFormula::Mayhew => Some(100.0 * weight / (52.2 + 41.9 * (-0.055 * r).exp())),
            OneRmFormula::OConner => Some(weight * (1.0 + r / 40.0)),
            OneRmFormula::Wathan => Some(weight * 100.0 / (48.8 + 53.8 * (-0.075 * r).exp())),
            OneRmFormula::Lander => {
                let denom = 1.013 - 0.0267123 * r;
                if denom <= 0.0 {
                    None
                } else {
                    Some(weight / denom)
                }
            }
        }
    }
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

/// Metric to use when building a histogram.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HistogramMetric {
    /// Histogram of set weights.
    Weight { bin: f64 },
    /// Histogram of training volume (weight * reps).
    Volume { bin: f64 },
    /// Histogram of RPE values.
    Rpe { bin: f64 },
    /// Histogram of repetition counts.
    Reps { bin: f64 },
}

/// Slice of a pie chart with metadata.
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub start: f64,
    pub sweep: f64,
    pub color: Color32,
}

/// Simple pie chart representation.
pub struct PieChart {
    pub slices: Vec<PieSlice>,
}

/// Result of generating a plot line with an optional marker for the maximum value.
#[derive(Clone)]
pub struct Record {
    pub point: [f64; 2],
    pub date: NaiveDate,
    pub weight: f64,
    pub reps: u32,
}

/// Result of generating a plot line with an optional marker for the maximum value.
pub struct LineWithMarker {
    pub line: Line,
    pub points: Vec<[f64; 2]>,
    pub max_point: Option<[f64; 2]>,
    pub label: Option<String>,
    pub records: Vec<Record>,
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
        let mut records = Vec::new();
        let mut idx = 0usize;
        let mut max_val = f64::NEG_INFINITY;
        let mut max_point = None;
        let ex_norm = normalize_exercise(exercise);
        let mut filtered: Vec<&WorkoutEntry> = entries
            .iter()
            .filter(|e| normalize_exercise(&e.exercise) == ex_norm)
            .collect();
        if x_axis == XAxis::Date {
            filtered.sort_by_key(|e| NaiveDate::parse_from_str(&e.date, "%Y-%m-%d").ok());
        }
        for e in filtered {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    let x = match x_axis {
                        XAxis::Date => d.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    let f = unit.factor() as f64;
                    let y = match y_axis {
                        YAxis::Weight => e.weight.unwrap() as f64 * f,
                        YAxis::Volume => e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64,
                    };
                    let cmp = match y_axis {
                        YAxis::Weight => e.weight.unwrap() as f64 * f,
                        YAxis::Volume => y,
                    };
                    if cmp > max_val {
                        max_val = cmp;
                        max_point = Some([x, y]);
                        records.push(Record {
                            point: [x, y],
                            date: d,
                            weight: e.weight.unwrap() as f64 * f,
                            reps: e.reps.unwrap() as u32,
                        });
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
            label: Some(match y_axis {
                YAxis::Weight => "Max Weight".to_string(),
                YAxis::Volume => "Max Volume".to_string(),
            }),
            records,
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
                    label: None,
                    records: Vec::new(),
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
        let mut records = Vec::new();
        let mut idx = 0usize;
        let mut max_est = f64::NEG_INFINITY;
        let mut max_point = None;
        let ex_norm = normalize_exercise(exercise);
        let mut filtered: Vec<&WorkoutEntry> = entries
            .iter()
            .filter(|e| normalize_exercise(&e.exercise) == ex_norm)
            .collect();
        if x_axis == XAxis::Date {
            filtered.sort_by_key(|e| NaiveDate::parse_from_str(&e.date, "%Y-%m-%d").ok());
        }
        for e in filtered {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    let f = unit.factor() as f64;
                    let weight = e.weight.unwrap() as f64 * f;
                    let est = match formula.estimate(weight, e.reps.unwrap()) {
                        Some(v) => v,
                        None => continue,
                    };
                    let x = match x_axis {
                        XAxis::Date => d.num_days_from_ce() as f64,
                        XAxis::WorkoutIndex => idx as f64,
                    };
                    if est > max_est {
                        max_est = est;
                        max_point = Some([x, est]);
                        records.push(Record {
                            point: [x, est],
                            date: d,
                            weight,
                            reps: e.reps.unwrap() as u32,
                        });
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
            label: Some("Max 1RM".to_string()),
            records,
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
                    label: None,
                    records: Vec::new(),
                });
            }
        }
    }
    lines
}

/// Build a histogram of rep counts for the selected exercises.
///
/// The x-axis represents the number of reps performed and the bar height shows
/// how many sets used that rep count. When `exercises` is empty all entries are
/// considered. Entries outside the optional date range are skipped.
pub fn rep_histogram(
    entries: &[WorkoutEntry],
    exercises: &[String],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> BarChart {
    let map = aggregate_rep_counts(entries, exercises, start, end);
    let bars: Vec<Bar> = map
        .into_iter()
        .map(|(reps, count)| Bar::new(reps as f64, count as f64))
        .collect();
    BarChart::new(bars).name("Reps")
}

/// Build a histogram of the chosen `metric` across `entries`.
///
/// Only entries within the optional date range are considered. Weights and
/// volumes are converted using `unit`.
pub fn histogram(
    entries: &[WorkoutEntry],
    metric: HistogramMetric,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    unit: WeightUnit,
) -> BarChart {
    use std::collections::BTreeMap;

    let (bin_size, name) = match metric {
        HistogramMetric::Weight { bin } => (bin, "Weight"),
        HistogramMetric::Volume { bin } => (bin, "Volume"),
        HistogramMetric::Rpe { bin } => (bin, "RPE"),
        HistogramMetric::Reps { bin } => (bin, "Reps"),
    };
    let f = unit.factor() as f64;
    let mut map: BTreeMap<i64, usize> = BTreeMap::new();
    for e in entries {
        if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let val = match metric {
                    HistogramMetric::Weight { .. } => Some(e.weight.unwrap() as f64 * f),
                    HistogramMetric::Volume { .. } => {
                        Some(e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64)
                    }
                    HistogramMetric::Rpe { .. } => e.raw.rpe.map(|r| r as f64),
                    HistogramMetric::Reps { .. } => Some(e.reps.unwrap() as f64),
                };
                if let Some(v) = val {
                    if bin_size > 0.0 {
                        let idx = (v / bin_size).floor() as i64;
                        *map.entry(idx).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let bars: Vec<Bar> = map
        .into_iter()
        .map(|(idx, count)| {
            let x = idx as f64 * bin_size + bin_size / 2.0;
            Bar::new(x, count as f64).width(bin_size)
        })
        .collect();
    BarChart::new(bars).name(name)
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

/// Create a bar chart showing the distribution of sets by primary body part.
///
/// Entries outside the optional date range are ignored. The resulting chart
/// contains one bar per body part with the height equal to the number of sets.
pub fn body_part_distribution(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> (BarChart, Vec<String>) {
    use std::collections::BTreeMap;

    let map = aggregate_sets_by_body_part(entries, start, end);
    let mut bars = Vec::new();
    let mut body_parts = Vec::new();
    for (idx, (part, count)) in BTreeMap::from_iter(map).into_iter().enumerate() {
        bars.push(Bar::new(idx as f64, count as f64));
        body_parts.push(part);
    }
    (BarChart::new(bars).name("Body Parts"), body_parts)
}

/// Create a pie chart showing the distribution of sets by primary body part.
///
/// Returns a [`PieChart`] where each slice corresponds to a body part and
/// contains metadata about the slice.
pub fn body_part_pie(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> PieChart {
    use std::collections::BTreeMap;
    use std::f64::consts::TAU;

    let map = aggregate_sets_by_body_part(entries, start, end);
    let total: f64 = map.values().sum::<usize>() as f64;
    let mut angle = 0.0;
    let mut slices = Vec::new();
    let parts = map.len().max(1);
    for (idx, (part, count)) in BTreeMap::from_iter(map).into_iter().enumerate() {
        let sweep = if total > 0.0 {
            (count as f64 / total) * TAU
        } else {
            0.0
        };
        let color: Color32 = Hsva::new(idx as f32 / parts as f32, 0.8, 0.8, 1.0).into();
        slices.push(PieSlice {
            label: part,
            value: count as f64,
            start: angle,
            sweep,
            color,
        });
        angle += sweep;
    }
    PieChart { slices }
}

/// Draw a pie chart and return the label of any clicked slice.
pub fn draw_pie_chart(ui: &mut Ui, chart: &PieChart, size: Vec2) -> Option<String> {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let center = rect.center();
    let radius = rect.width().min(rect.height()) / 2.0;
    let painter = ui.painter();
    let total: f64 = chart.slices.iter().map(|s| s.value).sum();

    for slice in &chart.slices {
        let steps = ((slice.sweep.abs() / std::f64::consts::TAU) * 64.0).ceil() as usize;
        let steps = steps.max(2);
        let mut points: Vec<Pos2> = Vec::with_capacity(steps + 2);
        points.push(center);
        for i in 0..=steps {
            let ang = slice.start + slice.sweep * i as f64 / steps as f64;
            let x = center.x + radius * ang.cos() as f32;
            let y = center.y + radius * ang.sin() as f32;
            points.push(Pos2 { x, y });
        }
        painter.add(Shape::convex_polygon(points, slice.color, Stroke::NONE));

        let mid = slice.start + slice.sweep / 2.0;
        let is_small = slice.sweep.abs() < std::f64::consts::TAU * 0.05;
        let text_radius = if is_small { radius * 0.8 } else { radius * 0.5 };
        let pos = Pos2 {
            x: center.x + text_radius * mid.cos() as f32,
            y: center.y + text_radius * mid.sin() as f32,
        };
        let hsv = Hsva::from(slice.color);
        let text_color = if hsv.v > 0.5 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        let pct = if total > 0.0 {
            format!(" ({:.1}%)", slice.value / total * 100.0)
        } else {
            String::new()
        };
        let text = format!("{}{}", slice.label, pct);
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(14.0),
            text_color,
        );
    }

    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let v = pos - center;
            let dist = v.length();
            if dist <= radius {
                let mut ang = v.y.atan2(v.x) as f64;
                if ang < 0.0 {
                    ang += std::f64::consts::TAU;
                }
                for slice in &chart.slices {
                    if ang >= slice.start && ang < slice.start + slice.sweep {
                        return Some(slice.label.clone());
                    }
                }
            }
        }
    }

    None
}

/// Generate scatter points of weight versus repetitions for the selected
/// `exercises`.
///
/// All matching entries within the optional date range are included. Weights
/// are converted using `unit` and plotted on the x-axis with reps on the
/// y-axis.
pub const MAX_WEIGHT: f64 = 10_000.0;
pub const MAX_REPS: f64 = 1_000.0;

pub fn weight_reps_scatter(
    entries: &[WorkoutEntry],
    exercises: &[String],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    unit: WeightUnit,
) -> Points {
    let mut pts = Vec::new();
    let f = unit.factor() as f64;
    let normalized: Vec<String> = exercises.iter().map(|s| normalize_exercise(s)).collect();
    for e in entries {
        let ex_name = normalize_exercise(&e.exercise);
        if normalized.is_empty() || normalized.iter().any(|ex| ex == &ex_name) {
            if let Ok(d) = NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                    if let (Some(w), Some(r)) = (e.weight, e.reps) {
                        let w = w as f64 * f;
                        let r = r as f64;
                        if w.is_finite() && r.is_finite() && w > 0.0 && r > 0.0 {
                            pts.push([w.clamp(0.0, MAX_WEIGHT), r.clamp(0.0, MAX_REPS)]);
                        }
                    }
                }
            }
        }
    }
    pts.retain(|p| p[0].is_finite() && p[1].is_finite());
    if pts.is_empty() {
        Points::new(vec![[0.0, 0.0]])
            .color(Color32::TRANSPARENT)
            .name("Weight vs Reps")
    } else {
        Points::new(pts).name("Weight vs Reps")
    }
}

/// Draw a crosshair at the provided plot coordinates.
///
/// This is used to highlight the current hover position when exploring
/// scatter plots. The crosshair is drawn using light gray dashed lines so it
/// does not overwhelm the underlying data.
pub fn draw_crosshair(plot_ui: &mut PlotUi, pos: egui_plot::PlotPoint) {
    plot_ui.vline(VLine::new(pos.x).color(Color32::LIGHT_GRAY));
    plot_ui.hline(HLine::new(pos.y).color(Color32::LIGHT_GRAY));
}

/// Format a tooltip string showing weight, reps, and training volume for the
/// provided plot coordinates.
pub fn format_hover_text(pos: egui_plot::PlotPoint, unit: WeightUnit) -> String {
    let unit_label = match unit {
        WeightUnit::Kg => "kg",
        WeightUnit::Lbs => "lbs",
    };
    let weight = pos.x;
    let reps = pos.y;
    let volume = weight * reps;
    format!("Weight: {weight:.0} {unit_label}\nReps: {reps:.0}\nVolume: {volume:.0}")
}

/// Build a bar chart of weekly set counts and a line for weekly volume.
pub fn weekly_summary_plot(weeks: &[WeeklySummary], unit: WeightUnit) -> (BarChart, Line) {
    let bars: Vec<Bar> = weeks
        .iter()
        .enumerate()
        .map(|(idx, w)| {
            let mut bar = Bar::new(idx as f64, w.total_sets as f64);
            if w.over_threshold {
                bar = bar.fill(Color32::RED);
            }
            bar
        })
        .collect();
    let pts: Vec<[f64; 2]> = weeks
        .iter()
        .enumerate()
        .map(|(idx, w)| [idx as f64, w.total_volume as f64 * unit.factor() as f64])
        .collect();
    (
        BarChart::new(bars).name("Sets"),
        Line::new(PlotPoints::from(pts)).name("Volume"),
    )
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
                entry.0 += e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64; // volume
                entry.1 += e.weight.unwrap() as f64 * f; // total weight
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
                        entry.0 += e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64;
                        entry.1 += e.weight.unwrap() as f64 * f;
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
                        entry.0 += e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64;
                        entry.1 += e.weight.unwrap() as f64 * f;
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

/// Generate a simple trend line for the given set of points using
/// linear regression.
///
/// Returns a pair of points representing the start and end of the
/// regression line across the provided data range.
/// Create a trend line based on a slope and the mean of the provided points.
///
/// This is useful when the slope has already been calculated using a
/// regression method and we simply need the start and end coordinates for the
/// overlay line.
pub fn trend_line_points_from_slope(points: &[[f64; 2]], slope: f64) -> Vec<[f64; 2]> {
    if points.len() < 2 {
        return Vec::new();
    }

    let n = points.len() as f64;
    let mean_x: f64 = points.iter().map(|p| p[0]).sum::<f64>() / n;
    let mean_y: f64 = points.iter().map(|p| p[1]).sum::<f64>() / n;
    let intercept = mean_y - slope * mean_x;

    let x_start = points.first().unwrap()[0];
    let x_end = points.last().unwrap()[0];
    vec![
        [x_start, slope * x_start + intercept],
        [x_end, slope * x_end + intercept],
    ]
}

/// Generate a simple trend line for the given set of points using
/// linear regression.
pub fn trend_line_points(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if points.len() < 2 {
        return Vec::new();
    }

    let n = points.len() as f64;
    let mean_x: f64 = points.iter().map(|p| p[0]).sum::<f64>() / n;
    let mean_y: f64 = points.iter().map(|p| p[1]).sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;
    for p in points {
        num += (p[0] - mean_x) * (p[1] - mean_y);
        den += (p[0] - mean_x) * (p[0] - mean_x);
    }
    let slope = if den == 0.0 { 0.0 } else { num / den };
    trend_line_points_from_slope(points, slope)
}

/// Generate a forecast line extending `months_ahead` months beyond the last
/// data point using a known slope.
///
/// The provided `slope_per_month` should represent the change in the y-value
/// for each month. The returned vector contains two points: the last data point
/// and the projected future point. When `XAxis::Date` is used the projection
/// assumes 30 days per month when extending the x-value.
pub fn forecast_line_points(
    points: &[[f64; 2]],
    slope_per_month: f64,
    months_ahead: f64,
    x_axis: XAxis,
) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return Vec::new();
    }
    let last = points[points.len() - 1];
    if let Some(y) = linear_projection(
        last[1] as f32,
        Some(slope_per_month as f32),
        months_ahead as f32,
    )
    .map(|v| v as f64)
    {
        let x = match x_axis {
            XAxis::Date => last[0] + months_ahead * 30.0,
            XAxis::WorkoutIndex => last[0] + months_ahead,
        };
        vec![last, [x, y]]
    } else {
        Vec::new()
    }
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

/// Create a line plot of average RPE over time.
///
/// RPE values are averaged per day. Entries without an RPE value are ignored.
pub fn rpe_over_time_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    ma_window: Option<usize>,
    method: SmoothingMethod,
) -> Vec<Line> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<NaiveDate, (f32, usize)> = BTreeMap::new();
    for e in entries {
        if let (Some(rpe), Ok(d)) = (e.raw.rpe, NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")) {
            if start.map_or(true, |s| d >= s) && end.map_or(true, |e2| d <= e2) {
                let entry = map.entry(d).or_insert((0.0, 0));
                entry.0 += rpe;
                entry.1 += 1;
            }
        }
    }
    let mut points = Vec::new();
    match x_axis {
        XAxis::Date => {
            for (d, (sum, count)) in map {
                points.push([d.num_days_from_ce() as f64, (sum / count as f32) as f64]);
            }
        }
        XAxis::WorkoutIndex => {
            for (i, (_d, (sum, count))) in map.into_iter().enumerate() {
                points.push([i as f64, (sum / count as f32) as f64]);
            }
        }
    }
    let mut lines = Vec::new();
    lines.push(Line::new(PlotPoints::from(points.clone())).name("Avg RPE"));
    if let Some(w) = ma_window.filter(|w| *w > 1) {
        if points.len() > 1 {
            let smooth = match method {
                SmoothingMethod::SimpleMA => moving_average_points(&points, w),
                SmoothingMethod::EMA => {
                    let alpha = 2.0 / (w as f64 + 1.0);
                    ema_points(&points, alpha)
                }
            };
            lines.push(Line::new(PlotPoints::from(smooth)).name("Avg RPE MA"));
        }
    }
    lines
}

/// Create a line plot of average RPE over time with optional smoothing and
/// highlighting of the maximum value.
///
/// This is a thin wrapper around [`rpe_over_time_line`] that augments the
/// returned [`Line`]s with metadata needed by the UI.
pub fn average_rpe_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    ma_window: Option<usize>,
    method: SmoothingMethod,
) -> Vec<LineWithMarker> {
    use egui_plot::PlotItem;
    use std::cmp::Ordering;
    let lines = rpe_over_time_line(entries, start, end, x_axis, ma_window, method);
    let mut out = Vec::new();
    for (i, line) in lines.into_iter().enumerate() {
        let mut points = Vec::new();
        if let egui_plot::PlotGeometry::Points(pts) = line.geometry() {
            points = pts.iter().map(|p| [p.x, p.y]).collect();
        }
        let (max_point, label) = if i == 0 {
            let mp = points
                .iter()
                .cloned()
                .max_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(Ordering::Equal));
            (mp, Some("Max Avg RPE".to_string()))
        } else {
            (None, None)
        };
        out.push(LineWithMarker {
            line,
            points,
            max_point,
            label,
            records: Vec::new(),
        });
    }
    out
}

/// Create a line plot of training volume per primary body part.
///
/// Each body part is plotted separately. Entries outside the optional date
/// range or without a known body part are skipped. Volume can be aggregated
/// daily, weekly or monthly.
pub fn body_part_volume_line(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    unit: WeightUnit,
    agg: VolumeAggregation,
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
                let key_date = match agg {
                    VolumeAggregation::Daily => d,
                    VolumeAggregation::Weekly => NaiveDate::from_isoywd_opt(
                        d.iso_week().year(),
                        d.iso_week().week(),
                        chrono::Weekday::Mon,
                    )
                    .unwrap_or(d),
                    VolumeAggregation::Monthly => {
                        NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap_or(d)
                    }
                };
                *map.entry(part.to_string())
                    .or_default()
                    .entry(key_date)
                    .or_insert(0.0) += e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64;
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

/// Generate trend lines of training volume per primary body part.
///
/// Volume is aggregated by the specified period and converted to the desired
/// `unit`. Only entries within the optional `start` and `end` dates and with a
/// known body part are considered. Each returned [`Line`] represents the
/// trend for a single body part.
pub fn body_part_volume_trend(
    entries: &[WorkoutEntry],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    unit: WeightUnit,
    agg: VolumeAggregation,
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
                let key_date = match agg {
                    VolumeAggregation::Daily => d,
                    VolumeAggregation::Weekly => NaiveDate::from_isoywd_opt(
                        d.iso_week().year(),
                        d.iso_week().week(),
                        chrono::Weekday::Mon,
                    )
                    .unwrap_or(d),
                    VolumeAggregation::Monthly => {
                        NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap_or(d)
                    }
                };
                *map.entry(part.to_string())
                    .or_default()
                    .entry(key_date)
                    .or_insert(0.0) += e.weight.unwrap() as f64 * f * e.reps.unwrap() as f64;
            }
        }
    }

    let mut lines = Vec::new();
    for (part, day_map) in map {
        let mut points = Vec::new();
        for (d, vol) in day_map {
            points.push([d.num_days_from_ce() as f64, vol]);
        }
        let trend = trend_line_points(&points);
        if trend.len() == 2 {
            lines.push(Line::new(PlotPoints::from(trend)).name(format!("{part} Trend")));
        }
    }
    lines
}

/// Create a line plot of training volume for a single exercise.
///
/// Volume is aggregated according to `agg`.
pub fn exercise_volume_line(
    entries: &[WorkoutEntry],
    exercise: &str,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    x_axis: XAxis,
    unit: WeightUnit,
    agg: VolumeAggregation,
    ma_window: Option<usize>,
) -> Vec<Line> {
    let target = normalize_exercise(exercise);
    let filtered: Vec<WorkoutEntry> = entries
        .iter()
        .filter(|e| normalize_exercise(&e.exercise) == target)
        .cloned()
        .collect();
    let points = aggregated_volume_points(&filtered, start, end, x_axis, YAxis::Volume, unit, agg);
    let mut lines = Vec::new();
    lines.push(Line::new(PlotPoints::from(points.clone())).name(exercise));
    if let Some(w) = ma_window.filter(|w| *w > 1) {
        if points.len() > 1 {
            let ma_pts = moving_average_points(&points, w);
            lines.push(Line::new(PlotPoints::from(ma_pts)).name(format!("{exercise} MA")));
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
                weight: Some(100.0),
                reps: Some(5),
                raw: RawWorkoutRow {
                    rpe: Some(8.0),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: Some(80.0),
                reps: Some(5),
                raw: RawWorkoutRow {
                    rpe: Some(7.0),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Squat".into(),
                weight: Some(105.0),
                reps: Some(5),
                raw: RawWorkoutRow {
                    rpe: Some(9.0),
                    ..RawWorkoutRow::default()
                },
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
    fn scatter_skips_invalid_entries() {
        fn entry(weight: Option<f32>, reps: Option<u32>) -> WorkoutEntry {
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight,
                reps,
                raw: RawWorkoutRow::default(),
            }
        }

        let entries = vec![
            entry(Some(100.0), Some(10)),
            entry(Some(f32::NAN), Some(10)),
            entry(Some(-50.0), Some(5)),
            entry(Some(100.0), Some(0)),
        ];

        let pts = weight_reps_scatter(&entries, &[], None, None, WeightUnit::Lbs);
        let bounds = PlotItem::bounds(&pts);
        assert_eq!(bounds.min(), [100.0, 10.0]);
        assert_eq!(bounds.max(), [100.0, 10.0]);
    }

    #[test]
    fn scatter_clamps_extreme_values() {
        fn entry(weight: f32, reps: u32) -> WorkoutEntry {
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: Some(weight),
                reps: Some(reps),
                raw: RawWorkoutRow::default(),
            }
        }

        let entries = vec![entry(100.0, 10), entry(50_000.0, 5_000)];

        let pts = weight_reps_scatter(&entries, &[], None, None, WeightUnit::Lbs);
        let bounds = PlotItem::bounds(&pts);
        assert_eq!(bounds.min(), [100.0, 10.0]);
        assert_eq!(bounds.max(), [super::MAX_WEIGHT, super::MAX_REPS]);
    }

    #[test]
    fn scatter_filters_non_finite_values() {
        fn entry(weight: f32, reps: u32) -> WorkoutEntry {
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: Some(weight),
                reps: Some(reps),
                raw: RawWorkoutRow::default(),
            }
        }

        let entries = vec![entry(f32::NAN, 10), entry(100.0, 10)];

        let pts = weight_reps_scatter(&entries, &[], None, None, WeightUnit::Lbs);
        let bounds = PlotItem::bounds(&pts);
        assert_eq!(bounds.min(), [100.0, 10.0]);
        assert_eq!(bounds.max(), [100.0, 10.0]);
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

    #[test]
    fn test_trend_line_points() {
        let pts = vec![[0.0, 1.0], [1.0, 3.0], [2.0, 5.0]];
        let trend = trend_line_points(&pts);
        assert_eq!(trend, vec![[0.0, 1.0], [2.0, 5.0]]);
    }

    #[test]
    fn test_trend_line_points_from_slope() {
        let pts = vec![[0.0, 1.0], [1.0, 3.0], [2.0, 5.0]];
        // slope is 2.0 for this perfectly linear data
        let trend = trend_line_points_from_slope(&pts, 2.0);
        assert_eq!(trend, vec![[0.0, 1.0], [2.0, 5.0]]);
    }

    #[test]
    fn test_forecast_line_points() {
        let pts = vec![[0.0, 1.0], [30.0, 3.0]];
        let forecast = forecast_line_points(&pts, 1.0, 2.0, XAxis::Date);
        assert_eq!(forecast, vec![[30.0, 3.0], [90.0, 5.0]]);
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
    fn test_rpe_over_time_line() {
        let line = rpe_over_time_line(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![
            [d1.num_days_from_ce() as f64, 7.5],
            [d3.num_days_from_ce() as f64, 9.0],
        ];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_rpe_over_time_line_index() {
        let line = rpe_over_time_line(
            &sample_entries(),
            None,
            None,
            XAxis::WorkoutIndex,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
        let expected = vec![[0.0, 7.5], [1.0, 9.0]];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_rpe_over_time_line_start_filter() {
        let start = NaiveDate::parse_from_str("2024-01-02", "%Y-%m-%d").unwrap();
        let line = rpe_over_time_line(
            &sample_entries(),
            Some(start),
            None,
            XAxis::Date,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![[d3.num_days_from_ce() as f64, 9.0]];
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_rpe_over_time_line_end_filter() {
        let end = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let line = rpe_over_time_line(
            &sample_entries(),
            None,
            Some(end),
            XAxis::Date,
            None,
            SmoothingMethod::SimpleMA,
        )
        .into_iter()
        .next()
        .unwrap();
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let expected = vec![[d1.num_days_from_ce() as f64, 7.5]];
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
        assert_eq!(lw.label.as_deref(), Some("Max Weight"));
    }

    #[test]
    fn test_weight_over_time_volume_highlight() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Squat".into(),
                weight: Some(100.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Squat".into(),
                weight: Some(80.0),
                reps: Some(10),
                raw: RawWorkoutRow::default(),
            },
        ];
        let res = weight_over_time_line(
            &entries,
            &["Squat".to_string()],
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Volume,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        let lw = res.into_iter().next().unwrap();
        let expected = vec![[0.0, 500.0], [1.0, 800.0]];
        assert_eq!(line_points(lw.line), expected);
        assert_eq!(lw.max_point, Some([1.0, 800.0]));
        assert_eq!(lw.label.as_deref(), Some("Max Volume"));
        assert_eq!(
            lw.records.iter().map(|r| r.point).collect::<Vec<_>>(),
            vec![[0.0, 500.0], [1.0, 800.0]]
        );
    }

    #[test]
    fn test_record_points_weight() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: Some(100.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Bench".into(),
                weight: Some(90.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Bench".into(),
                weight: Some(110.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
        ];
        let res = weight_over_time_line(
            &entries,
            &["Bench".to_string()],
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Weight,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        let lw = res.into_iter().next().unwrap();
        assert_eq!(
            lw.records.iter().map(|r| r.point).collect::<Vec<_>>(),
            vec![[0.0, 100.0], [2.0, 110.0]]
        );
    }

    #[test]
    fn test_record_points_1rm() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Deadlift".into(),
                weight: Some(100.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Deadlift".into(),
                weight: Some(110.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Deadlift".into(),
                weight: Some(105.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
        ];
        let res = estimated_1rm_line(
            &entries,
            &["Deadlift".to_string()],
            OneRmFormula::Epley,
            None,
            None,
            XAxis::WorkoutIndex,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        let lw = res.into_iter().next().unwrap();
        assert_eq!(lw.records.len(), 2);
        assert_eq!(lw.records[0].point[0], 0.0);
        assert_eq!(lw.records[1].point[0], 1.0);
    }

    #[test]
    fn test_record_metadata() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: Some(100.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-03".into(),
                exercise: "Bench".into(),
                weight: Some(110.0),
                reps: Some(5),
                raw: RawWorkoutRow::default(),
            },
        ];
        let res = weight_over_time_line(
            &entries,
            &["Bench".to_string()],
            None,
            None,
            XAxis::WorkoutIndex,
            YAxis::Weight,
            WeightUnit::Lbs,
            None,
            SmoothingMethod::SimpleMA,
        );
        let lw = res.into_iter().next().unwrap();
        assert_eq!(lw.records.len(), 2);
        assert_eq!(
            lw.records[0].date,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        assert_eq!(lw.records[0].weight, 100.0);
        assert_eq!(lw.records[0].reps, 5);
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
    fn test_body_part_volume_line_weekly() {
        let lines = body_part_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            VolumeAggregation::Weekly,
            None,
        );
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let expected = vec![
            vec![[d.num_days_from_ce() as f64, 400.0]],
            vec![[d.num_days_from_ce() as f64, 1025.0]],
        ];
        for (l, exp) in lines.into_iter().zip(expected) {
            assert_eq!(line_points(l), exp);
        }
    }

    #[test]
    fn test_body_part_volume_line_monthly() {
        let lines = body_part_volume_line(
            &sample_entries(),
            None,
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            VolumeAggregation::Monthly,
            None,
        );
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let expected = vec![
            vec![[d.num_days_from_ce() as f64, 400.0]],
            vec![[d.num_days_from_ce() as f64, 1025.0]],
        ];
        for (l, exp) in lines.into_iter().zip(expected) {
            assert_eq!(line_points(l), exp);
        }
    }

    #[test]
    fn test_body_part_volume_trend() {
        let lines = body_part_volume_trend(
            &sample_entries(),
            None,
            None,
            WeightUnit::Lbs,
            VolumeAggregation::Daily,
        );
        // Only the "Quads" body part has two data points, so only one trend line is returned
        assert_eq!(lines.len(), 1);
        let d1 = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let d3 = NaiveDate::parse_from_str("2024-01-03", "%Y-%m-%d").unwrap();
        let expected = vec![
            [d1.num_days_from_ce() as f64, 500.0],
            [d3.num_days_from_ce() as f64, 525.0],
        ];
        let pts = line_points(lines.into_iter().next().unwrap());
        assert_eq!(pts, expected);
    }

    #[test]
    fn test_exercise_volume_line_weekly() {
        let lines = exercise_volume_line(
            &sample_entries(),
            "squat",
            None,
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            VolumeAggregation::Weekly,
            None,
        );
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let expected = vec![[d.num_days_from_ce() as f64, 1025.0]];
        let line = lines.into_iter().next().unwrap();
        assert_eq!(line_points(line), expected);
    }

    #[test]
    fn test_exercise_volume_line_monthly() {
        let lines = exercise_volume_line(
            &sample_entries(),
            "BENCH",
            None,
            None,
            XAxis::Date,
            WeightUnit::Lbs,
            VolumeAggregation::Monthly,
            None,
        );
        let d = NaiveDate::parse_from_str("2024-01-01", "%Y-%m-%d").unwrap();
        let expected = vec![[d.num_days_from_ce() as f64, 400.0]];
        let line = lines.into_iter().next().unwrap();
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
    fn test_rep_histogram_bounds() {
        let entries = sample_entries();
        let ex = vec!["Squat".to_string(), "Bench".to_string()];
        let chart = rep_histogram(&entries, &ex, None, None);
        assert!(matches!(chart.geometry(), PlotGeometry::Rects));
        let bounds = PlotItem::bounds(&chart);
        assert!((bounds.max()[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_histogram_counts() {
        let entries = sample_entries();
        let chart = histogram(
            &entries,
            HistogramMetric::Weight { bin: 10.0 },
            None,
            None,
            WeightUnit::Lbs,
        );
        assert!(matches!(chart.geometry(), PlotGeometry::Rects));
        let bounds = PlotItem::bounds(&chart);
        // Two entries fall into the same 10 lb bin (100-110).
        assert!((bounds.max()[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_body_part_distribution_counts() {
        use crate::analysis::aggregate_sets_by_body_part;
        use std::collections::HashMap;

        let entries = sample_entries();
        let (chart, _) = body_part_distribution(&entries, None, None);
        let bounds = PlotItem::bounds(&chart);

        let expected =
            HashMap::from([("Chest".to_string(), 1usize), ("Quads".to_string(), 2usize)]);

        assert_eq!(aggregate_sets_by_body_part(&entries, None, None), expected);
        assert!((bounds.max()[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_body_part_pie_counts() {
        let entries = sample_entries();
        let pie = body_part_pie(&entries, None, None);
        let mut map = std::collections::HashMap::new();
        for s in &pie.slices {
            map.insert(s.label.clone(), s.value as usize);
        }
        assert_eq!(map.get("Chest"), Some(&1));
        assert_eq!(map.get("Quads"), Some(&2));
        let total_sweep: f64 = pie.slices.iter().map(|s| s.sweep).sum();
        assert!((total_sweep - std::f64::consts::TAU).abs() < 1e-6);
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
        assert_eq!(lw_e.label.as_deref(), Some("Max 1RM"));

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
        assert_eq!(lw_b.label.as_deref(), Some("Max 1RM"));
    }

    #[test]
    fn test_one_rm_formula_estimate() {
        let w = 200.0;
        let reps = 5;
        let lombardi = OneRmFormula::Lombardi
            .estimate(w, reps)
            .expect("lombardi should produce value");
        let expected_l = w * (reps as f64).powf(0.10);
        assert!((lombardi - expected_l).abs() < 1e-6);

        let mayhew = OneRmFormula::Mayhew
            .estimate(w, reps)
            .expect("mayhew should produce value");
        let expected_m = 100.0 * w / (52.2 + 41.9 * (-0.055 * reps as f64).exp());
        assert!((mayhew - expected_m).abs() < 1e-6);

        let oconner = OneRmFormula::OConner
            .estimate(w, reps)
            .expect("oconner should produce value");
        let expected_o = w * (1.0 + reps as f64 / 40.0);
        assert!((oconner - expected_o).abs() < 1e-6);

        let wathan = OneRmFormula::Wathan
            .estimate(w, reps)
            .expect("wathan should produce value");
        let expected_w = 100.0 * w / (48.8 + 53.8 * (-0.075 * reps as f64).exp());
        assert!((wathan - expected_w).abs() < 1e-6);

        let lander = OneRmFormula::Lander
            .estimate(w, reps)
            .expect("lander should produce value");
        let expected_la = w / (1.013 - 0.0267123 * reps as f64);
        assert!((lander - expected_la).abs() < 1e-6);

        // invalid inputs
        assert!(OneRmFormula::Brzycki.estimate(w, 37).is_none());
        assert!(OneRmFormula::Lander.estimate(w, 38).is_none());
    }
}
