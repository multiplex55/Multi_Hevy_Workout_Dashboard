//! Main application logic and persistent user settings.

use dirs_next as dirs;
use eframe::{App, Frame, NativeOptions, egui};
use egui_extras::DatePickerButton;
use egui_plot::{Legend, Line, MarkerShape, Plot, PlotGeometry, PlotItem, PlotPoints, Points};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Cursor;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate};
use log::info;

mod analysis;
use analysis::{BasicStats, ExerciseStats, compute_stats, format_load_message};
mod plotting;
use plotting::{
    OneRmFormula, SmoothingMethod, VolumeAggregation, XAxis, YAxis, aggregated_volume_points,
    body_part_distribution, body_part_volume_line, estimated_1rm_line, exercise_volume_line,
    rep_histogram, rpe_over_time_line, sets_per_day_bar, training_volume_line, trend_line_points,
    unique_exercises, weekly_summary_plot, weight_over_time_line,
};
mod capture;
use capture::{crop_image, save_png};
mod export;
use export::{save_entries_csv, save_entries_json, save_stats_csv, save_stats_json};
mod body_parts;
use body_parts::ExerciseType;
mod exercise_mapping;

#[derive(Debug, Deserialize, Clone, Serialize)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
    raw: RawWorkoutRow,
}

impl WorkoutEntry {
    fn body_part(&self) -> Option<String> {
        body_parts::body_part_for(&self.exercise)
    }

    fn exercise_type(&self) -> Option<ExerciseType> {
        body_parts::info_for(&self.exercise).map(|i| i.kind)
    }

    fn difficulty(&self) -> Option<body_parts::Difficulty> {
        body_parts::difficulty_for(&self.exercise)
    }

    fn equipment(&self) -> Option<body_parts::Equipment> {
        body_parts::equipment_for(&self.exercise)
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
struct RawWorkoutRow {
    title: Option<String>,
    start_time: String,
    end_time: Option<String>,
    description: Option<String>,
    exercise_title: String,
    superset_id: Option<String>,
    exercise_notes: Option<String>,
    set_index: Option<u32>,
    set_type: Option<String>,
    weight_lbs: Option<f32>,
    weight_kg: Option<f32>,
    reps: Option<u32>,
    distance_miles: Option<f32>,
    duration_seconds: Option<f32>,
    rpe: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeightUnit {
    Lbs,
    Kg,
}

impl WeightUnit {
    fn factor(self) -> f32 {
        match self {
            WeightUnit::Lbs => 1.0,
            WeightUnit::Kg => 0.453_592,
        }
    }
}

fn parse_workout_csv<R: std::io::Read>(reader: R) -> Result<Vec<WorkoutEntry>, csv::Error> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut entries = Vec::new();
    for result in rdr.deserialize::<RawWorkoutRow>() {
        if let Ok(raw) = result {
            if let Ok(dt) =
                chrono::NaiveDateTime::parse_from_str(&raw.start_time, "%d %b %Y, %H:%M")
            {
                let date = dt.date().format("%Y-%m-%d").to_string();
                let weight_lbs = raw
                    .weight_lbs
                    .or_else(|| raw.weight_kg.map(|kg| kg * 2.20462));
                let weight = weight_lbs.unwrap_or(0.0);
                let reps = raw.reps.unwrap_or(0);
                entries.push(WorkoutEntry {
                    date,
                    exercise: raw.exercise_title.clone(),
                    weight,
                    reps,
                    raw,
                });
            }
        }
    }
    Ok(entries)
}

fn parse_latest_pr_number(json: &str) -> Option<u64> {
    #[derive(serde::Deserialize)]
    struct Pr {
        number: u64,
    }
    serde_json::from_str::<Vec<Pr>>(json)
        .ok()?
        .into_iter()
        .next()
        .map(|p| p.number)
}

fn check_for_new_pr(repo: &str, last: Option<u64>) -> Option<u64> {
    let url = format!("https://api.github.com/repos/{repo}/pulls?per_page=1");
    let resp = ureq::get(&url).set("User-Agent", "multi-hevy").call();
    if let Ok(r) = resp {
        if r.status() == 200 {
            if let Ok(text) = r.into_string() {
                if let Some(num) = parse_latest_pr_number(&text) {
                    if Some(num) != last {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

fn default_plot_width() -> f32 {
    400.0
}

fn default_plot_height() -> f32 {
    200.0
}

/// Persistent configuration for user preferences and plot visibility.
///
/// The values are serialized to a JSON file so choices like `show_rpe`
/// survive across application restarts.  `show_rpe` controls the visibility
/// of the RPE plot and is marked with `#[serde(default)]`, causing it to
/// default to `false` when the field is absent from an older configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Settings {
    show_weight: bool,
    show_est_1rm: bool,
    show_sets: bool,
    #[serde(default)]
    show_rep_histogram: bool,
    show_volume: bool,
    /// Controls visibility of the RPE plot.
    ///
    /// This field uses `#[serde(default)]` so that missing values in the
    /// configuration file default to `false`.
    #[serde(default)]
    show_rpe: bool,
    #[serde(default)]
    show_rpe_trend: bool,
    show_body_part_volume: bool,
    #[serde(default)]
    show_body_part_distribution: bool,
    #[serde(default)]
    show_exercise_volume: bool,
    #[serde(default)]
    show_weekly_summary: bool,
    #[serde(default)]
    show_exercise_stats: bool,
    #[serde(default)]
    show_personal_records: bool,
    #[serde(default)]
    show_exercise_panel: bool,
    highlight_max: bool,
    #[serde(default)]
    show_weight_trend: bool,
    #[serde(default)]
    show_volume_trend: bool,
    show_smoothed: bool,
    ma_window: usize,
    smoothing_method: SmoothingMethod,
    #[serde(default = "default_plot_width")]
    plot_width: f32,
    #[serde(default = "default_plot_height")]
    plot_height: f32,
    #[serde(default)]
    volume_aggregation: VolumeAggregation,
    #[serde(default)]
    body_part_volume_aggregation: VolumeAggregation,
    weight_unit: WeightUnit,
    one_rm_formula: OneRmFormula,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
    set_type_filter: Option<String>,
    superset_filter: Option<String>,
    body_part_filter: Option<String>,
    exercise_type_filter: Option<ExerciseType>,
    difficulty_filter: Option<body_parts::Difficulty>,
    equipment_filter: Option<body_parts::Equipment>,
    min_rpe: Option<f32>,
    max_rpe: Option<f32>,
    min_weight: Option<f32>,
    max_weight: Option<f32>,
    min_reps: Option<u32>,
    max_reps: Option<u32>,
    notes_filter: Option<String>,
    #[serde(default)]
    exclude_warmups: bool,
    #[serde(default)]
    auto_load_last: bool,
    last_file: Option<String>,
    #[serde(default)]
    check_prs: bool,
    github_repo: Option<String>,
    last_pr: Option<u64>,
    selected_exercises: Vec<String>,
    table_filter: String,
    sort_column: SortColumn,
    sort_ascending: bool,
    summary_sort: SummarySort,
    summary_sort_ascending: bool,
}

impl Settings {
    const FILE: &'static str = "multi_hevy_settings.json";

    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join(Self::FILE))
    }

    /// Load settings from the JSON configuration file.
    ///
    /// Missing fields, including `show_rpe`, default to `false` or their
    /// respective values thanks to `#[serde(default)]` on the struct fields.
    fn load() -> Self {
        if let Some(path) = Self::path() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    /// Persist the current settings, including the `show_rpe` flag, to disk.
    fn save(&self) {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, data);
            }
        }
    }
}

impl Default for Settings {
    /// Construct a `Settings` instance with default values.
    ///
    /// By default, the RPE plot is hidden (`show_rpe` is `false`).
    fn default() -> Self {
        Self {
            show_weight: true,
            show_est_1rm: true,
            show_sets: true,
            show_rep_histogram: false,
            show_volume: false,
            show_rpe: false,
            show_rpe_trend: false,
            show_body_part_volume: false,
            show_body_part_distribution: false,
            show_exercise_volume: false,
            show_weekly_summary: false,
            show_exercise_stats: false,
            show_personal_records: false,
            show_exercise_panel: true,
            highlight_max: true,
            show_weight_trend: false,
            show_volume_trend: false,
            show_smoothed: false,
            smoothing_method: SmoothingMethod::SimpleMA,
            ma_window: 5,
            plot_width: 400.0,
            plot_height: 200.0,
            volume_aggregation: VolumeAggregation::Weekly,
            body_part_volume_aggregation: VolumeAggregation::Weekly,
            weight_unit: WeightUnit::Lbs,
            one_rm_formula: OneRmFormula::Epley,
            start_date: None,
            end_date: None,
            x_axis: XAxis::Date,
            y_axis: YAxis::Weight,
            set_type_filter: None,
            superset_filter: None,
            body_part_filter: None,
            exercise_type_filter: None,
            difficulty_filter: None,
            equipment_filter: None,
            min_rpe: None,
            max_rpe: None,
            min_weight: None,
            max_weight: None,
            min_reps: None,
            max_reps: None,
            notes_filter: None,
            exclude_warmups: false,
            auto_load_last: true,
            last_file: None,
            check_prs: false,
            github_repo: None,
            last_pr: None,
            selected_exercises: Vec::new(),
            table_filter: String::new(),
            sort_column: SortColumn::Date,
            sort_ascending: true,
            summary_sort: SummarySort::Exercise,
            summary_sort_ascending: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SortColumn {
    Date,
    Exercise,
    Weight,
    Reps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SummarySort {
    Exercise,
    Sets,
    Reps,
    Volume,
    MaxWeight,
    Best1Rm,
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
    stats: BasicStats,
    selected_exercises: Vec<String>,
    set_types: Vec<String>,
    superset_ids: Vec<String>,
    search_query: String,
    table_filter: String,
    last_loaded: Option<String>,
    toast_start: Option<Instant>,
    settings: Settings,
    show_settings: bool,
    show_entries: bool,
    show_plot_window: bool,
    show_compare_window: bool,
    show_exercise_stats: bool,
    show_personal_records: bool,
    show_exercise_panel: bool,
    show_about: bool,
    sort_column: SortColumn,
    sort_ascending: bool,
    summary_sort: SummarySort,
    summary_sort_ascending: bool,
    capture_rect: Option<egui::Rect>,
    settings_dirty: bool,
    show_mapping: bool,
    mapping_exercise: String,
    mapping_dirty: bool,
    mapping_entry: exercise_mapping::MuscleMapping,
    pr_toast_start: Option<Instant>,
    pr_message: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        let settings = Settings::load();
        exercise_mapping::load();
        let show_exercise_stats = settings.show_exercise_stats;
        let show_personal_records = settings.show_personal_records;
        let show_exercise_panel = settings.show_exercise_panel;
        let mut app = Self {
            workouts: Vec::new(),
            stats: BasicStats::default(),
            selected_exercises: Vec::new(),
            set_types: Vec::new(),
            superset_ids: Vec::new(),
            search_query: String::new(),
            table_filter: String::new(),
            last_loaded: None,
            toast_start: None,
            settings,
            show_settings: false,
            show_entries: false,
            show_plot_window: false,
            show_compare_window: false,
            show_exercise_stats,
            show_personal_records,
            show_exercise_panel,
            show_about: false,
            sort_column: SortColumn::Date,
            sort_ascending: true,
            summary_sort: SummarySort::Exercise,
            summary_sort_ascending: true,
            capture_rect: None,
            settings_dirty: false,
            show_mapping: false,
            mapping_exercise: String::new(),
            mapping_dirty: false,
            mapping_entry: exercise_mapping::MuscleMapping::default(),
            pr_toast_start: None,
            pr_message: None,
        };

        app.selected_exercises = app.settings.selected_exercises.clone();
        app.table_filter = app.settings.table_filter.clone();
        app.sort_column = app.settings.sort_column;
        app.sort_ascending = app.settings.sort_ascending;
        app.summary_sort = app.settings.summary_sort;
        app.summary_sort_ascending = app.settings.summary_sort_ascending;

        if app.settings.auto_load_last {
            if let Some(path) = app.settings.last_file.clone() {
                let p = std::path::Path::new(&path);
                if p.exists() {
                    if let Ok(file) = File::open(p) {
                        if let Ok(entries) = parse_workout_csv(file) {
                            app.workouts = entries;
                            app.stats = compute_stats(
                                &app.workouts,
                                app.settings.start_date,
                                app.settings.end_date,
                            );
                            app.update_filter_values();
                            app.last_loaded =
                                p.file_name().map(|f| f.to_string_lossy().to_string());
                            app.toast_start = Some(Instant::now());
                        }
                    }
                }
            }
        }

        if app.settings.check_prs {
            if let Some(ref repo) = app.settings.github_repo {
                if let Some(new_id) = check_for_new_pr(repo, app.settings.last_pr) {
                    app.pr_message = Some(format!("New PR #{new_id} available"));
                    app.pr_toast_start = Some(Instant::now());
                    app.settings.last_pr = Some(new_id);
                }
            }
        }

        app.update_filter_values();

        app
    }
}

impl MyApp {
    fn sort_button(
        ui: &mut egui::Ui,
        label: &str,
        column: SortColumn,
        sort_column: &mut SortColumn,
        sort_ascending: &mut bool,
    ) {
        let arrow = if *sort_column == column {
            if *sort_ascending {
                " \u{25B2}"
            } else {
                " \u{25BC}"
            }
        } else {
            ""
        };
        if ui.button(format!("{label}{arrow}")).clicked() {
            if *sort_column == column {
                *sort_ascending = !*sort_ascending;
            } else {
                *sort_column = column;
                *sort_ascending = true;
            }
        }
    }

    fn summary_sort_button(
        ui: &mut egui::Ui,
        label: &str,
        column: SummarySort,
        sort_column: &mut SummarySort,
        sort_ascending: &mut bool,
    ) {
        let arrow = if *sort_column == column {
            if *sort_ascending {
                " \u{25B2}"
            } else {
                " \u{25BC}"
            }
        } else {
            ""
        };
        if ui.button(format!("{label}{arrow}")).clicked() {
            if *sort_column == column {
                *sort_ascending = !*sort_ascending;
            } else {
                *sort_column = column;
                *sort_ascending = true;
            }
        }
    }

    fn sort_summary_stats(
        stats: &mut Vec<(String, ExerciseStats)>,
        sort: SummarySort,
        ascending: bool,
    ) {
        stats.sort_by(|a, b| {
            use std::cmp::Ordering;
            let ord = match sort {
                SummarySort::Exercise => a.0.cmp(&b.0),
                SummarySort::Sets => a.1.total_sets.cmp(&b.1.total_sets),
                SummarySort::Reps => a.1.total_reps.cmp(&b.1.total_reps),
                SummarySort::Volume => {
                    a.1.total_volume
                        .partial_cmp(&b.1.total_volume)
                        .unwrap_or(Ordering::Equal)
                }
                SummarySort::MaxWeight => {
                    a.1.max_weight
                        .unwrap_or(0.0)
                        .partial_cmp(&b.1.max_weight.unwrap_or(0.0))
                        .unwrap_or(Ordering::Equal)
                }
                SummarySort::Best1Rm => {
                    a.1.best_est_1rm
                        .unwrap_or(0.0)
                        .partial_cmp(&b.1.best_est_1rm.unwrap_or(0.0))
                        .unwrap_or(Ordering::Equal)
                }
            };
            let ord = if ord == Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                ord
            };
            if ascending { ord } else { ord.reverse() }
        });
    }

    fn trend_value(trend: Option<f32>, factor: f32) -> String {
        match trend {
            Some(t) if t > 0.0 => format!("+{:.1}", t * factor),
            Some(t) if t < 0.0 => format!("\u{2013}{:.1}", (t * factor).abs()),
            _ => "\u{2013}".to_owned(),
        }
    }

    fn update_filter_values(&mut self) {
        self.set_types = analysis::unique_set_types(&self.workouts);
        self.superset_ids = analysis::unique_superset_ids(&self.workouts);
    }

    fn entry_matches_filters(&self, e: &WorkoutEntry) -> bool {
        if self.settings.exclude_warmups {
            if e.raw
                .set_type
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("warmup"))
                == Some(true)
            {
                return false;
            }
        }
        if let Some(ref st) = self.settings.set_type_filter {
            if e.raw
                .set_type
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case(st))
                != Some(true)
            {
                return false;
            }
        }
        if let Some(ref ss) = self.settings.superset_filter {
            if e.raw
                .superset_id
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case(ss))
                != Some(true)
            {
                return false;
            }
        }
        if let Some(min) = self.settings.min_rpe {
            if e.raw.rpe.map(|r| r < min).unwrap_or(true) {
                return false;
            }
        }
        if let Some(max) = self.settings.max_rpe {
            if e.raw.rpe.map(|r| r > max).unwrap_or(true) {
                return false;
            }
        }
        if let Some(min_w) = self.settings.min_weight {
            if e.weight < min_w {
                return false;
            }
        }
        if let Some(max_w) = self.settings.max_weight {
            if e.weight > max_w {
                return false;
            }
        }
        if let Some(min_r) = self.settings.min_reps {
            if e.reps < min_r {
                return false;
            }
        }
        if let Some(max_r) = self.settings.max_reps {
            if e.reps > max_r {
                return false;
            }
        }
        if let Some(ref nf) = self.settings.notes_filter {
            let nf_l = nf.to_lowercase();
            if e.raw
                .exercise_notes
                .as_deref()
                .map(|n| n.to_lowercase().contains(&nf_l))
                != Some(true)
            {
                return false;
            }
        }
        if let Some(ref bp) = self.settings.body_part_filter {
            match e.body_part() {
                Some(p) if p.eq_ignore_ascii_case(bp) => {}
                _ => return false,
            }
        }
        if let Some(kind) = self.settings.exercise_type_filter {
            match e.exercise_type() {
                Some(k) if k == kind => {}
                _ => return false,
            }
        }
        if let Some(diff) = self.settings.difficulty_filter {
            match e.difficulty() {
                Some(d) if d == diff => {}
                _ => return false,
            }
        }
        if let Some(eq) = self.settings.equipment_filter {
            match e.equipment() {
                Some(d) if d == eq => {}
                _ => return false,
            }
        }
        true
    }

    fn filtered_entries(&self) -> Vec<WorkoutEntry> {
        self.workouts
            .iter()
            .filter(|e| self.entry_matches_filters(e))
            .cloned()
            .collect()
    }

    fn filtered_entry_refs(&self) -> Vec<&WorkoutEntry> {
        self.workouts
            .iter()
            .filter(|e| self.entry_matches_filters(e))
            .collect()
    }

    /// Return entries that match the current filters and the selected exercises.
    fn filtered_selected_entries(&self) -> Vec<WorkoutEntry> {
        self.filtered_entries()
            .into_iter()
            .filter(|e| {
                self.selected_exercises.is_empty() || self.selected_exercises.contains(&e.exercise)
            })
            .collect()
    }

    /// Recompute `self.stats` using only the selected exercises.
    fn update_selected_stats(&mut self) {
        let entries = self.filtered_selected_entries();
        self.stats = compute_stats(&entries, self.settings.start_date, self.settings.end_date);
    }

    fn exercise_set_counts(&self, exercise: &str) -> (usize, usize, usize) {
        use std::collections::HashSet;
        let mut workouts = HashSet::new();
        let mut working = 0usize;
        let mut warmups = 0usize;
        for e in self.filtered_entry_refs() {
            if e.exercise.eq_ignore_ascii_case(exercise) {
                let id = format!(
                    "{}{}",
                    e.raw.title.as_deref().unwrap_or(""),
                    e.raw.start_time
                );
                workouts.insert(id);
                if e.raw
                    .set_type
                    .as_deref()
                    .map(|s| s.eq_ignore_ascii_case("warmup"))
                    == Some(true)
                {
                    warmups += 1;
                } else {
                    working += 1;
                }
            }
        }
        (workouts.len(), working, warmups)
    }

    fn draw_plot(
        &self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        filtered: &[WorkoutEntry],
        sel: &[String],
    ) -> egui_plot::PlotResponse<()> {
        let x_axis = self.settings.x_axis;
        let mut first_resp: Option<egui_plot::PlotResponse<()>> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                if self.settings.show_weight {
                    let mut all_points: Vec<[f64; 2]> = Vec::new();
                    let mut highlight: Option<[f64; 2]> = None;
                    let resp = Plot::new("weight_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pointer = plot_ui.pointer_coordinate();
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            for lw in weight_over_time_line(
                                filtered,
                                sel,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.y_axis,
                                self.settings.weight_unit,
                                ma,
                                self.settings.smoothing_method,
                            ) {
                                for p in &lw.points {
                                    all_points.push(*p);
                                }
                                plot_ui.line(lw.line);
                                if self.settings.show_weight_trend && lw.max_point.is_some() {
                                    let trend = trend_line_points(&lw.points);
                                    if trend.len() == 2 {
                                        plot_ui
                                            .line(Line::new(PlotPoints::from(trend)).name("Trend"));
                                    }
                                }
                                if self.settings.highlight_max {
                                    if let (Some(p), Some(label)) =
                                        (lw.max_point, lw.label.as_deref())
                                    {
                                        let color = match label {
                                            "Max Weight" => egui::Color32::RED,
                                            "Max Volume" => egui::Color32::GREEN,
                                            _ => egui::Color32::WHITE,
                                        };
                                        plot_ui.points(
                                            Points::new(vec![p])
                                                .shape(MarkerShape::Diamond)
                                                .color(color)
                                                .name(label),
                                        );
                                    }
                                    if !lw.record_points.is_empty() {
                                        plot_ui.points(
                                            Points::new(lw.record_points.clone())
                                                .shape(MarkerShape::Asterisk)
                                                .color(egui::Color32::LIGHT_GREEN)
                                                .name("Record"),
                                        );
                                    }
                                }
                            }
                            if let Some(ptr) = pointer {
                                if let Some(p) = nearest_point(ptr, &all_points) {
                                    highlight = Some(p);
                                    plot_ui.points(
                                        Points::new(vec![p])
                                            .color(egui::Color32::YELLOW)
                                            .highlight(true)
                                            .name("Hovered"),
                                    );
                                }
                            }
                        });

                    if let Some(p) = highlight {
                        if resp.response.hovered() {
                            egui::show_tooltip_at_pointer(
                                ctx,
                                egui::Id::new("plot_tip_weight"),
                                |ui| {
                                    let x_text = match self.settings.x_axis {
                                        XAxis::Date => {
                                            NaiveDate::from_num_days_from_ce_opt(p[0] as i32)
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| format!("{}", p[0] as i64))
                                        }
                                        XAxis::WorkoutIndex => format!("{}", p[0] as i64),
                                    };
                                    ui.label(format!("{x_text}: {:.2}", p[1]));
                                },
                            );
                        }
                    }

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_est_1rm {
                    let mut all_points: Vec<[f64; 2]> = Vec::new();
                    let mut highlight: Option<[f64; 2]> = None;
                    let resp = Plot::new("est_1rm_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pointer = plot_ui.pointer_coordinate();
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            for lr in estimated_1rm_line(
                                filtered,
                                sel,
                                self.settings.one_rm_formula,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.weight_unit,
                                ma,
                                self.settings.smoothing_method,
                            ) {
                                for p in &lr.points {
                                    all_points.push(*p);
                                }
                                plot_ui.line(lr.line);
                                if self.settings.highlight_max {
                                    if let (Some(p), Some(label)) =
                                        (lr.max_point, lr.label.as_deref())
                                    {
                                        let (shape, color) = match label {
                                            "Max 1RM" => (MarkerShape::Circle, egui::Color32::BLUE),
                                            "Max Weight" => {
                                                (MarkerShape::Diamond, egui::Color32::RED)
                                            }
                                            "Max Volume" => {
                                                (MarkerShape::Diamond, egui::Color32::GREEN)
                                            }
                                            _ => (MarkerShape::Circle, egui::Color32::WHITE),
                                        };
                                        plot_ui.points(
                                            Points::new(vec![p])
                                                .shape(shape)
                                                .color(color)
                                                .name(label),
                                        );
                                    }
                                    if !lr.record_points.is_empty() {
                                        plot_ui.points(
                                            Points::new(lr.record_points.clone())
                                                .shape(MarkerShape::Asterisk)
                                                .color(egui::Color32::LIGHT_GREEN)
                                                .name("Record"),
                                        );
                                    }
                                }
                            }
                            if let Some(ptr) = pointer {
                                if let Some(p) = nearest_point(ptr, &all_points) {
                                    highlight = Some(p);
                                    plot_ui.points(
                                        Points::new(vec![p])
                                            .color(egui::Color32::YELLOW)
                                            .highlight(true)
                                            .name("Hovered"),
                                    );
                                }
                            }
                        });

                    if let Some(p) = highlight {
                        if resp.response.hovered() {
                            egui::show_tooltip_at_pointer(
                                ctx,
                                egui::Id::new("plot_tip_1rm"),
                                |ui| {
                                    let x_text = match self.settings.x_axis {
                                        XAxis::Date => {
                                            NaiveDate::from_num_days_from_ce_opt(p[0] as i32)
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| format!("{}", p[0] as i64))
                                        }
                                        XAxis::WorkoutIndex => format!("{}", p[0] as i64),
                                    };
                                    ui.label(format!("{x_text}: {:.2}", p[1]));
                                },
                            );
                        }
                    }

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_volume {
                    let mut all_points: Vec<[f64; 2]> = Vec::new();
                    let mut highlight: Option<[f64; 2]> = None;
                    let resp = Plot::new("volume_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pointer = plot_ui.pointer_coordinate();
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            let mut raw_points: Vec<[f64; 2]> = Vec::new();
                            for (idx, l) in training_volume_line(
                                filtered,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.y_axis,
                                self.settings.weight_unit,
                                ma,
                                self.settings.smoothing_method,
                            )
                            .into_iter()
                            .enumerate()
                            {
                                if let PlotGeometry::Points(pts) = l.geometry() {
                                    for p in pts {
                                        all_points.push([p.x, p.y]);
                                        if idx == 0 {
                                            raw_points.push([p.x, p.y]);
                                        }
                                    }
                                }
                                plot_ui.line(l);
                            }
                            if self.settings.show_volume_trend && raw_points.len() > 1 {
                                let trend = trend_line_points(&raw_points);
                                if trend.len() == 2 {
                                    plot_ui.line(Line::new(PlotPoints::from(trend)).name("Trend"));
                                }
                            }
                            if self.settings.volume_aggregation != VolumeAggregation::Daily {
                                let pts = aggregated_volume_points(
                                    filtered,
                                    self.settings.start_date,
                                    self.settings.end_date,
                                    self.settings.x_axis,
                                    self.settings.y_axis,
                                    self.settings.weight_unit,
                                    self.settings.volume_aggregation,
                                );
                                let name = match self.settings.volume_aggregation {
                                    VolumeAggregation::Weekly => "Weekly Volume",
                                    VolumeAggregation::Monthly => "Monthly Volume",
                                    VolumeAggregation::Daily => "Daily Volume",
                                };
                                for p in &pts {
                                    all_points.push(*p);
                                }
                                plot_ui.line(Line::new(PlotPoints::from(pts)).name(name));
                            }
                            if let Some(ptr) = pointer {
                                if let Some(p) = nearest_point(ptr, &all_points) {
                                    highlight = Some(p);
                                    plot_ui.points(
                                        Points::new(vec![p])
                                            .color(egui::Color32::YELLOW)
                                            .highlight(true)
                                            .name("Hovered"),
                                    );
                                }
                            }
                        });

                    if let Some(p) = highlight {
                        if resp.response.hovered() {
                            egui::show_tooltip_at_pointer(
                                ctx,
                                egui::Id::new("plot_tip_vol"),
                                |ui| {
                                    let x_text = match self.settings.x_axis {
                                        XAxis::Date => {
                                            NaiveDate::from_num_days_from_ce_opt(p[0] as i32)
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| format!("{}", p[0] as i64))
                                        }
                                        XAxis::WorkoutIndex => format!("{}", p[0] as i64),
                                    };
                                    ui.label(format!("{x_text}: {:.2}", p[1]));
                                },
                            );
                        }
                    }

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_exercise_volume && sel.len() == 1 {
                    let resp = Plot::new("exercise_volume_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            for l in exercise_volume_line(
                                filtered,
                                &sel[0],
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.weight_unit,
                                self.settings.volume_aggregation,
                                ma,
                            ) {
                                plot_ui.line(l);
                            }
                        });

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_body_part_volume {
                    let resp = Plot::new("body_part_volume_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            for l in body_part_volume_line(
                                filtered,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.weight_unit,
                                self.settings.body_part_volume_aggregation,
                                ma,
                            ) {
                                plot_ui.line(l);
                            }
                        });

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_body_part_distribution {
                    let resp = Plot::new("body_part_distribution_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| format!("{:.0}", mark.value))
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(body_part_distribution(
                                filtered,
                                self.settings.start_date,
                                self.settings.end_date,
                            ));
                        });

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_sets {
                    let ex_for_sets = if sel.len() == 1 {
                        Some(sel[0].as_str())
                    } else {
                        None
                    };
                    let resp = Plot::new("sets_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| format!("{:.0}", mark.value))
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(sets_per_day_bar(
                                filtered,
                                ex_for_sets,
                                self.settings.start_date,
                                self.settings.end_date,
                            ));
                        });

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_rep_histogram {
                    let resp = Plot::new("rep_hist_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| format!("{:.0}", mark.value))
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(rep_histogram(
                                filtered,
                                sel,
                                self.settings.start_date,
                                self.settings.end_date,
                            ));
                        });

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_rpe {
                    let mut all_points: Vec<[f64; 2]> = Vec::new();
                    let mut line_points: Vec<[f64; 2]> = Vec::new();
                    let mut highlight: Option<[f64; 2]> = None;
                    let resp = Plot::new("rpe_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            if x_axis == XAxis::Date {
                                NaiveDate::from_num_days_from_ce_opt(mark.value.round() as i32)
                                    .map(|d| d.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| format!("{:.0}", mark.value))
                            } else {
                                format!("{:.0}", mark.value)
                            }
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            let pointer = plot_ui.pointer_coordinate();
                            let ma = if self.settings.show_smoothed {
                                Some(self.settings.ma_window)
                            } else {
                                None
                            };
                            for (i, l) in rpe_over_time_line(
                                filtered,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                ma,
                                self.settings.smoothing_method,
                            )
                            .into_iter()
                            .enumerate()
                            {
                                if let PlotGeometry::Points(pts) = l.geometry() {
                                    for p in pts {
                                        let arr = [p.x, p.y];
                                        all_points.push(arr);
                                        if i == 0 {
                                            line_points.push(arr);
                                        }
                                    }
                                }
                                plot_ui.line(l);
                            }
                            if self.settings.show_rpe_trend {
                                let trend = trend_line_points(&line_points);
                                if trend.len() == 2 {
                                    plot_ui.line(Line::new(PlotPoints::from(trend)).name("Trend"));
                                }
                            }
                            if let Some(ptr) = pointer {
                                if let Some(p) = nearest_point(ptr, &all_points) {
                                    highlight = Some(p);
                                    plot_ui.points(
                                        Points::new(vec![p])
                                            .color(egui::Color32::YELLOW)
                                            .highlight(true)
                                            .name("Hovered"),
                                    );
                                }
                            }
                        });

                    if let Some(p) = highlight {
                        if resp.response.hovered() {
                            egui::show_tooltip_at_pointer(
                                ctx,
                                egui::Id::new("plot_tip_rpe"),
                                |ui| {
                                    let x_text = match self.settings.x_axis {
                                        XAxis::Date => {
                                            NaiveDate::from_num_days_from_ce_opt(p[0] as i32)
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| format!("{}", p[0] as i64))
                                        }
                                        XAxis::WorkoutIndex => format!("{}", p[0] as i64),
                                    };
                                    ui.label(format!("{x_text}: {:.2}", p[1]));
                                },
                            );
                        }
                    }

                    first_resp.get_or_insert(resp);
                }

                if self.settings.show_weekly_summary {
                    let weeks = analysis::aggregate_weekly_summary(
                        filtered,
                        self.settings.start_date,
                        self.settings.end_date,
                    );
                    let weeks_for_axis = weeks.clone();
                    let (bars, line) = weekly_summary_plot(&weeks, self.settings.weight_unit);
                    let resp = Plot::new("weekly_summary_plot")
                        .width(self.settings.plot_width)
                        .height(self.settings.plot_height)
                        .x_axis_formatter(move |mark, _chars, _| {
                            let idx = mark.value.round() as usize;
                            weeks_for_axis
                                .get(idx)
                                .map(|w| format!("{}-{:02}", w.year, w.week))
                                .unwrap_or_else(|| format!("{:.0}", mark.value))
                        })
                        .legend(Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(bars);
                            plot_ui.line(line);
                        });

                    first_resp.get_or_insert(resp);
                    let f = self.settings.weight_unit.factor();
                    egui::Grid::new("weekly_summary_table")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Year");
                            ui.label("Week");
                            ui.label("Sets");
                            ui.label("Volume");
                            ui.end_row();
                            for w in &weeks {
                                ui.label(w.year.to_string());
                                ui.label(format!("{:02}", w.week));
                                ui.label(w.total_sets.to_string());
                                ui.label(format!("{:.1}", w.total_volume * f));
                                ui.end_row();
                            }
                        });
                }
            });
        });

        first_resp.unwrap_or_else(|| {
            Plot::new("empty_plot")
                .width(self.settings.plot_width)
                .height(self.settings.plot_height)
                .show(ui, |_ui| {})
        })
    }

    fn sync_settings_from_app(&mut self) {
        self.settings.selected_exercises = self.selected_exercises.clone();
        self.settings.table_filter = self.table_filter.clone();
        self.settings.sort_column = self.sort_column;
        self.settings.sort_ascending = self.sort_ascending;
        self.settings.summary_sort = self.summary_sort;
        self.settings.summary_sort_ascending = self.summary_sort_ascending;
        self.settings.show_exercise_stats = self.show_exercise_stats;
        self.settings.show_personal_records = self.show_personal_records;
        self.settings.show_exercise_panel = self.show_exercise_panel;
    }
}

fn nearest_point(pointer: egui_plot::PlotPoint, points: &[[f64; 2]]) -> Option<[f64; 2]> {
    points.iter().copied().min_by(|a, b| {
        let da = (a[0] - pointer.x).powi(2) + (a[1] - pointer.y).powi(2);
        let db = (b[0] - pointer.x).powi(2) + (b[1] - pointer.y).powi(2);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Handle screenshot results
        let mut shot: Option<std::sync::Arc<egui::ColorImage>> = None;
        ctx.input_mut(|i| {
            i.events.retain(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    shot = Some(image.clone());
                    false
                } else {
                    true
                }
            });
        });
        if let Some(img) = shot {
            if let Some(rect) = self.capture_rect.take() {
                if let Some(path) = FileDialog::new().add_filter("PNG", &["png"]).save_file() {
                    let cropped = crop_image(&img, rect, ctx.pixels_per_point());
                    if let Err(err) = save_png(&cropped, &path) {
                        log::error!("Failed to save plot: {err}");
                    }
                }
            }
        }

        // Handle CSV drag-and-drop
        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            let ext_ok = file
                .path
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("csv"))
                .unwrap_or_else(|| file.name.to_lowercase().ends_with(".csv"));
            if !ext_ok {
                continue;
            }

            if let Some(path) = file.path.clone() {
                if let Ok(f) = File::open(&path) {
                    if let Ok(entries) = parse_workout_csv(f) {
                        self.workouts = entries;
                    } else {
                        self.workouts.clear();
                    }
                    let filename = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.display().to_string());
                    info!("Loaded {} entries from {}", self.workouts.len(), filename);
                    self.stats = compute_stats(
                        &self.workouts,
                        self.settings.start_date,
                        self.settings.end_date,
                    );
                    self.update_filter_values();
                    self.last_loaded = Some(filename);
                    self.toast_start = Some(Instant::now());
                    self.settings.last_file = Some(path.display().to_string());
                    self.settings_dirty = true;
                }
            } else if let Some(bytes) = file.bytes {
                let name = file.name.clone();
                let reader = Cursor::new(bytes.to_vec());
                if let Ok(entries) = parse_workout_csv(reader) {
                    self.workouts = entries;
                } else {
                    self.workouts.clear();
                }
                info!("Loaded {} entries from {}", self.workouts.len(), name);
                self.stats = compute_stats(
                    &self.workouts,
                    self.settings.start_date,
                    self.settings.end_date,
                );
                self.update_filter_values();
                self.last_loaded = Some(name);
                self.toast_start = Some(Instant::now());
                self.settings_dirty = true;
            }
        }
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                    if ui.button("Raw Entries").clicked() {
                        self.show_entries = true;
                        ui.close_menu();
                    }
                    if ui.button("Exercise Stats").clicked() {
                        self.show_exercise_stats = !self.show_exercise_stats;
                        self.settings.show_exercise_stats = self.show_exercise_stats;
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Personal Records").clicked() {
                        self.show_personal_records = !self.show_personal_records;
                        self.settings.show_personal_records = self.show_personal_records;
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Exercise Panel").clicked() {
                        self.show_exercise_panel = !self.show_exercise_panel;
                        self.settings.show_exercise_panel = self.show_exercise_panel;
                        self.settings_dirty = true;
                        ui.close_menu();
                    }
                    if ui.button("Muscle Mapping").clicked() {
                        self.show_mapping = true;
                        ui.close_menu();
                    }
                    if ui.button("Usage Tips").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                    if ui.button("Export Stats").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("CSV", &["csv"])
                            .save_file()
                        {
                            let mut exercises = analysis::aggregate_exercise_stats(
                                &self.workouts,
                                self.settings.one_rm_formula,
                                self.settings.start_date,
                                self.settings.end_date,
                            )
                            .into_iter()
                            .collect::<Vec<_>>();
                            MyApp::sort_summary_stats(
                                &mut exercises,
                                self.summary_sort,
                                self.summary_sort_ascending,
                            );
                            match path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s.to_lowercase())
                            {
                                Some(ext) if ext == "csv" => {
                                    if let Err(e) = save_stats_csv(&path, &self.stats, &exercises) {
                                        log::error!("Failed to export stats: {e}");
                                    }
                                }
                                _ => {
                                    if let Err(e) = save_stats_json(&path, &self.stats, &exercises)
                                    {
                                        log::error!("Failed to export stats: {e}");
                                    }
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Export Entries").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("CSV", &["csv"])
                            .save_file()
                        {
                            match path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|s| s.to_lowercase())
                            {
                                Some(ext) if ext == "csv" => {
                                    if let Err(e) = save_entries_csv(&path, &self.workouts) {
                                        log::error!("Failed to export entries: {e}");
                                    }
                                }
                                _ => {
                                    if let Err(e) = save_entries_json(&path, &self.workouts) {
                                        log::error!("Failed to export entries: {e}");
                                    }
                                }
                            }
                        }
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("control_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load CSV").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                        let filename = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.display().to_string());
                        if let Ok(file) = File::open(&path) {
                            if let Ok(entries) = parse_workout_csv(file) {
                                self.workouts = entries;
                            } else {
                                self.workouts.clear();
                            }
                            info!("Loaded {} entries from {}", self.workouts.len(), filename);
                            self.stats = compute_stats(
                                &self.workouts,
                                self.settings.start_date,
                                self.settings.end_date,
                            );
                            self.update_filter_values();
                            self.last_loaded = Some(filename);
                            self.toast_start = Some(Instant::now());
                            self.settings.last_file = Some(path.display().to_string());
                            self.settings_dirty = true;
                        }
                    }
                }

                if !self.workouts.is_empty() {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.search_query);

                    let filtered = self.filtered_entries();
                    let mut exercises = unique_exercises(
                        &filtered,
                        self.settings.start_date,
                        self.settings.end_date,
                    );
                    if !self.search_query.is_empty() {
                        let q = self.search_query.to_lowercase();
                        exercises.retain(|e| e.to_lowercase().contains(&q));
                    }

                    ui.label("Exercises:");
                    let resp = ui.menu_button(
                        if self.selected_exercises.is_empty() {
                            "Select Exercises".to_string()
                        } else {
                            self.selected_exercises.join(", ")
                        },
                        |ui| {
                            for ex in &exercises {
                                let mut sel = self.selected_exercises.contains(ex);
                                if ui.checkbox(&mut sel, ex).changed() {
                                    if sel {
                                        if !self.selected_exercises.contains(ex) {
                                            self.selected_exercises.push(ex.clone());
                                        }
                                    } else {
                                        self.selected_exercises.retain(|e| e != ex);
                                    }
                                    self.update_selected_stats();
                                }
                            }
                        },
                    );
                    let _ = ctx.input(|i| i.pointer.interact_pos());
                    resp.response.context_menu(|ui| {
                        if ui.button("Clear selection").clicked() {
                            self.selected_exercises.clear();
                            self.update_selected_stats();
                            ui.close_menu();
                        }
                        for ex in self.selected_exercises.clone() {
                            let label = format!("Remove {ex}");
                            if ui.button(label).clicked() {
                                self.selected_exercises.retain(|e| e != &ex);
                                self.update_selected_stats();
                                ui.close_menu();
                            }
                        }
                    });

                    if ui.button("Clear Exercises").clicked() {
                        self.selected_exercises.clear();
                        self.settings.selected_exercises.clear();
                        self.settings_dirty = true;
                        self.update_selected_stats();
                    }
                    // Plot action buttons moved to control bar
                }
            });
        });
        egui::SidePanel::left("info_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.workouts.is_empty() {
                    ui.label("No CSV loaded");
                    ui.label("Load a CSV to begin");
                } else {
                    ui.label(format!("Loaded {} entries", self.workouts.len()));
                }

                ui.separator();
                if self.selected_exercises.is_empty() {
                    ui.label("No exercises selected");
                    ui.label("Select exercises from the dropdown");
                } else {
                    ui.label(format!("Selected: {}", self.selected_exercises.join(", ")));
                }

                ui.separator();
                ui.collapsing("Available plots", |ui| {
                    ui.label("• Weight over time");
                    ui.label("• Estimated 1RM");
                    ui.label("• Sets per day");
                    ui.label("• Training volume");
                    ui.label("• Volume by body part");
                });
            });
        });

        if self.show_exercise_panel {
            egui::SidePanel::right("exercise_panel").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if !self.workouts.is_empty() {
                        ui.heading("Exercise Summary");
                        let filtered = self.filtered_entries();
                        let mut stats = analysis::aggregate_exercise_stats(
                            &filtered,
                            self.settings.one_rm_formula,
                            self.settings.start_date,
                            self.settings.end_date,
                        )
                        .into_iter()
                        .collect::<Vec<_>>();
                        MyApp::sort_summary_stats(
                            &mut stats,
                            self.summary_sort,
                            self.summary_sort_ascending,
                        );
                        let mut summary_sort = self.summary_sort;
                        let mut summary_sort_ascending = self.summary_sort_ascending;
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("exercise_summary_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Exercise",
                                        SummarySort::Exercise,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Sets",
                                        SummarySort::Sets,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Reps",
                                        SummarySort::Reps,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Volume",
                                        SummarySort::Volume,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Max Weight",
                                        SummarySort::MaxWeight,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    MyApp::summary_sort_button(
                                        ui,
                                        "Best 1RM",
                                        SummarySort::Best1Rm,
                                        &mut summary_sort,
                                        &mut summary_sort_ascending,
                                    );
                                    ui.label("Weight Trend");
                                    ui.label("Volume Trend");
                                    ui.end_row();
                                    MyApp::sort_summary_stats(
                                        &mut stats,
                                        summary_sort,
                                        summary_sort_ascending,
                                    );
                                    for (ex, s) in stats {
                                        ui.label(ex);
                                        ui.label(s.total_sets.to_string());
                                        ui.label(s.total_reps.to_string());
                                        let f = self.settings.weight_unit.factor();
                                        ui.label(format!("{:.1}", s.total_volume * f));
                                        if let Some(w) = s.max_weight {
                                            ui.label(format!("{:.1}", w * f));
                                        } else {
                                            ui.label("-");
                                        }
                                        if let Some(b) = s.best_est_1rm {
                                            ui.label(format!("{:.1}", b * f));
                                        } else {
                                            ui.label("-");
                                        }
                                        ui.label(MyApp::trend_value(s.weight_trend, f));
                                        ui.label(MyApp::trend_value(s.volume_trend, f));
                                        ui.end_row();
                                    }
                                });
                        });
                        self.summary_sort = summary_sort;
                        self.summary_sort_ascending = summary_sort_ascending;
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.workouts.is_empty() {
                ui.heading("Workout Statistics");
                if self.settings.start_date.is_some() || self.settings.end_date.is_some() {
                    let start = self
                        .settings
                        .start_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "start".into());
                    let end = self
                        .settings
                        .end_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "end".into());
                    ui.label(format!("Range: {start} - {end}"));
                }
                ui.label(format!("Total workouts: {}", self.stats.total_workouts));
                ui.label(format!(
                    "Avg sets/workout: {:.2}",
                    self.stats.avg_sets_per_workout
                ));
                ui.label(format!("Avg reps/set: {:.2}", self.stats.avg_reps_per_set));
                ui.label(format!(
                    "Avg days between: {:.2}",
                    self.stats.avg_days_between
                ));
                if let Some(ref ex) = self.stats.most_common_exercise {
                    ui.label(format!("Most common exercise: {}", ex));
                }
                if ui.button("Open Table").clicked() {
                    self.show_entries = true;
                }
                ui.separator();

                let filtered = self.filtered_entries();

                if self.selected_exercises.is_empty() {
                    ui.label("No exercises selected");
                } else {
                    let sel: Vec<String> = self.selected_exercises.clone();
                    let plot_resp = self.draw_plot(ctx, ui, &filtered, &sel);
                    if ui.button("Save Plot").clicked() {
                        self.capture_rect = Some(plot_resp.response.rect);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                    }
                    if ui.button("Clear Exercises").clicked() {
                        self.selected_exercises.clear();
                        self.settings.selected_exercises.clear();
                        self.settings_dirty = true;
                        self.update_selected_stats();
                    }
                    let _ = ctx.input(|i| i.pointer.interact_pos());
                    plot_resp.response.context_menu(|ui| {
                        if ui.button("Export CSV").clicked() {
                            if let Some(path) =
                                FileDialog::new().add_filter("CSV", &["csv"]).save_file()
                            {
                                let entries: Vec<WorkoutEntry> = self
                                    .filtered_entries()
                                    .into_iter()
                                    .filter(|e| self.selected_exercises.contains(&e.exercise))
                                    .collect();
                                if let Err(e) = save_entries_csv(&path, &entries) {
                                    log::error!("Failed to export entries: {e}");
                                }
                            }
                            ui.close_menu();
                        }
                        if ui.button("Remove exercise").clicked() {
                            for ex in &sel {
                                self.selected_exercises.retain(|e| e != ex);
                            }
                            self.update_selected_stats();
                            ui.close_menu();
                        }
                    });
                    if ui.button("Plot Window").clicked() {
                        self.show_plot_window = !self.show_plot_window;
                    }
                    if ui.button("Compare Window").clicked() {
                        self.show_compare_window = !self.show_compare_window;
                    }
                    if ui.button("Exercise Stats").clicked() {
                        self.show_exercise_stats = !self.show_exercise_stats;
                        self.settings.show_exercise_stats = self.show_exercise_stats;
                        self.settings_dirty = true;
                    }
                    if ui.button("Exercise Panel").clicked() {
                        self.show_exercise_panel = !self.show_exercise_panel;
                        self.settings.show_exercise_panel = self.show_exercise_panel;
                        self.settings_dirty = true;
                    }
                }
            }

            ui.heading("Workout Entries");
        });

        if self.show_entries {
            let mut entries = self.filtered_entries();
            let mut table_filter = self.table_filter.clone();
            let mut sort_column = self.sort_column;
            let mut sort_ascending = self.sort_ascending;
            egui::Window::new("Workout Entries")
                .open(&mut self.show_entries)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filter exercise:");
                        ui.text_edit_singleline(&mut table_filter);
                    });
                    if !table_filter.is_empty() {
                        let q = table_filter.to_lowercase();
                        entries.retain(|e| e.exercise.to_lowercase().contains(&q));
                    }
                    if let Some(start) = self.settings.start_date {
                        entries.retain(|e| {
                            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                                .map(|d| d >= start)
                                .unwrap_or(false)
                        });
                    }
                    if let Some(end) = self.settings.end_date {
                        entries.retain(|e| {
                            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                                .map(|d| d <= end)
                                .unwrap_or(false)
                        });
                    }
                    entries.sort_by(|a, b| match sort_column {
                        SortColumn::Date => a.date.cmp(&b.date),
                        SortColumn::Exercise => a.exercise.cmp(&b.exercise),
                        SortColumn::Weight => a
                            .weight
                            .partial_cmp(&b.weight)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        SortColumn::Reps => a.reps.cmp(&b.reps),
                    });
                    if !sort_ascending {
                        entries.reverse();
                    }
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(egui_extras::Column::auto())
                        .column(egui_extras::Column::auto())
                        .column(egui_extras::Column::auto())
                        .column(egui_extras::Column::auto())
                        .header(row_height, |mut header| {
                            header.col(|ui| {
                                MyApp::sort_button(
                                    ui,
                                    "Date",
                                    SortColumn::Date,
                                    &mut sort_column,
                                    &mut sort_ascending,
                                )
                            });
                            header.col(|ui| {
                                MyApp::sort_button(
                                    ui,
                                    "Exercise",
                                    SortColumn::Exercise,
                                    &mut sort_column,
                                    &mut sort_ascending,
                                );
                            });
                            header.col(|ui| {
                                MyApp::sort_button(
                                    ui,
                                    "Weight",
                                    SortColumn::Weight,
                                    &mut sort_column,
                                    &mut sort_ascending,
                                )
                            });
                            header.col(|ui| {
                                MyApp::sort_button(
                                    ui,
                                    "Reps",
                                    SortColumn::Reps,
                                    &mut sort_column,
                                    &mut sort_ascending,
                                )
                            });
                        })
                        .body(|mut body| {
                            for e in entries {
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&e.date);
                                    });
                                    row.col(|ui| {
                                        ui.label(&e.exercise);
                                    });
                                    row.col(|ui| {
                                        let f = self.settings.weight_unit.factor();
                                        ui.label(format!("{:.1}", e.weight * f));
                                    });
                                    row.col(|ui| {
                                        ui.label(e.reps.to_string());
                                    });
                                });
                            }
                        });
                });
            self.table_filter = table_filter;
            self.sort_column = sort_column;
            self.sort_ascending = sort_ascending;
        }

        if self.show_plot_window {
            let filtered = self.filtered_entries();
            let mut open = self.show_plot_window;
            egui::Window::new("Plot Window")
                .open(&mut open)
                .vscroll(true)
                .show(ctx, |ui| {
                    if self.selected_exercises.is_empty() {
                        ui.label("No exercises selected");
                    } else {
                        let sel: Vec<String> = self.selected_exercises.clone();
                        let plot_resp = self.draw_plot(ctx, ui, &filtered, &sel);
                        if ui.button("Save Plot").clicked() {
                            self.capture_rect = Some(plot_resp.response.rect);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                        }
                    }
                });
            self.show_plot_window = open;
        }

        if self.show_compare_window {
            let filtered = self.filtered_entries();
            let mut open = self.show_compare_window;
            egui::Window::new("Exercise Comparison")
                .open(&mut open)
                .vscroll(true)
                .show(ctx, |ui| {
                    if self.selected_exercises.is_empty() {
                        ui.label("No exercises selected");
                    } else {
                        let sel = self.selected_exercises.clone();
                        ui.columns(sel.len(), |cols| {
                            for (col, ex) in cols.iter_mut().zip(sel.iter()) {
                                self.draw_plot(ctx, col, &filtered, &[ex.clone()]);
                            }
                        });
                    }
                });
            self.show_compare_window = open;
        }

        if self.show_exercise_stats {
            let mut open = self.show_exercise_stats;
            egui::Window::new("Exercise Stats")
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    let entries = self.filtered_selected_entries();
                    let stats_map = analysis::aggregate_exercise_stats(
                        &entries,
                        self.settings.one_rm_formula,
                        self.settings.start_date,
                        self.settings.end_date,
                    );
                    let rec_map = analysis::personal_records(
                        &entries,
                        self.settings.one_rm_formula,
                        self.settings.start_date,
                        self.settings.end_date,
                    );
                    let mut rep_counts: BTreeSet<u32> = BTreeSet::new();
                    for rec in rec_map.values() {
                        for reps in rec.rep_prs.keys() {
                            rep_counts.insert(*reps);
                        }
                    }
                    let f = self.settings.weight_unit.factor();
                    egui::Grid::new("exercise_stats_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Exercise");
                            ui.label("Sets");
                            ui.label("Reps");
                            ui.label("Volume");
                            ui.label("Max Weight");
                            ui.label("Best 1RM");
                            ui.label("Weight Trend");
                            ui.label("Volume Trend");
                            ui.end_row();
                            for ex in &self.selected_exercises {
                                if let Some(s) = stats_map.get(ex) {
                                    ui.label(ex);
                                    ui.label(s.total_sets.to_string());
                                    ui.label(s.total_reps.to_string());
                                    ui.label(format!("{:.1}", s.total_volume * f));
                                    if let Some(w) = s.max_weight {
                                        ui.label(format!("{:.1}", w * f));
                                    } else {
                                        ui.label("-");
                                    }
                                    if let Some(b) = s.best_est_1rm {
                                        ui.label(format!("{:.1}", b * f));
                                    } else {
                                        ui.label("-");
                                    }
                                    ui.label(MyApp::trend_value(s.weight_trend, f));
                                    ui.label(MyApp::trend_value(s.volume_trend, f));
                                    ui.end_row();
                                }
                            }
                        });
                    if !rep_counts.is_empty() {
                        ui.separator();
                        egui::Grid::new("rep_pr_grid").striped(true).show(ui, |ui| {
                            ui.label("Exercise");
                            for reps in &rep_counts {
                                ui.label(reps.to_string());
                            }
                            ui.end_row();
                            for ex in &self.selected_exercises {
                                ui.label(ex);
                                if let Some(rec) = rec_map.get(ex) {
                                    for reps in &rep_counts {
                                        if let Some(w) = rec.rep_prs.get(reps) {
                                            ui.label(format!("{:.1}", w * f));
                                        } else {
                                            ui.label("-");
                                        }
                                    }
                                } else {
                                    for _ in &rep_counts {
                                        ui.label("-");
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    }
                });
            self.show_exercise_stats = open;
        }

        if self.show_personal_records {
            let mut open = self.show_personal_records;
            egui::Window::new("Personal Records")
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    let entries = self.filtered_entries();
                    let mut recs: Vec<_> = analysis::personal_records(
                        &entries,
                        self.settings.one_rm_formula,
                        self.settings.start_date,
                        self.settings.end_date,
                    )
                    .into_iter()
                    .collect();
                    recs.sort_by(|a, b| a.0.cmp(&b.0));
                    let f = self.settings.weight_unit.factor();
                    egui::Grid::new("personal_records_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Exercise");
                            ui.label("Max Weight");
                            ui.label("Max Volume");
                            ui.label("Best 1RM");
                            ui.end_row();
                            for (ex, r) in &recs {
                                ui.label(ex);
                                if let Some(w) = r.max_weight {
                                    ui.label(format!("{:.1}", w * f));
                                } else {
                                    ui.label("-");
                                }
                                if let Some(v) = r.max_volume {
                                    ui.label(format!("{:.1}", v * f));
                                } else {
                                    ui.label("-");
                                }
                                if let Some(b) = r.best_est_1rm {
                                    ui.label(format!("{:.1}", b * f));
                                } else {
                                    ui.label("-");
                                }
                                ui.end_row();
                            }
                        });
                });
            self.show_personal_records = open;
        }

        if self.show_about {
            egui::Window::new("Usage Tips")
                .open(&mut self.show_about)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Multi Hevy Workout Dashboard");
                    ui.separator();
                    ui.label("\u{2022} Load a CSV export from Hevy using \"Load CSV\".");
                    ui.label("\u{2022} Select exercises from the drop-down to update plots.");
                    ui.label("\u{2022} Configure which plots are shown in the Settings window.");
                    ui.label("\u{2022} Open Raw Entries from the File menu to view all sets.");
                });
        }

        if self.show_mapping {
            let mut open = self.show_mapping;
            egui::Window::new("Muscle Mapping")
                .default_width(400.0)
                .default_height(300.0)
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    let list = unique_exercises(&self.workouts, None, None);
                    let mut selected = self.mapping_exercise.clone();
                    egui::ComboBox::from_id_source("map_exercise_combo")
                        .selected_text(if selected.is_empty() {
                            "Select".into()
                        } else {
                            selected.clone()
                        })
                        .show_ui(ui, |ui| {
                            for e in &list {
                                ui.selectable_value(&mut selected, e.clone(), e);
                            }
                        });
                    if selected != self.mapping_exercise {
                        self.mapping_exercise = selected.clone();
                        self.mapping_entry = exercise_mapping::get(&selected).unwrap_or_default();
                    }
                    if !self.mapping_exercise.is_empty() {
                        let muscles = body_parts::primary_muscle_groups();
                        egui::ComboBox::from_id_source("map_primary")
                            .selected_text(if self.mapping_entry.primary.is_empty() {
                                "Select"
                            } else {
                                &self.mapping_entry.primary
                            })
                            .show_ui(ui, |ui| {
                                for m in &muscles {
                                    ui.selectable_value(
                                        &mut self.mapping_entry.primary,
                                        m.clone(),
                                        m,
                                    );
                                }
                            });
                        ui.label("Secondary:");
                        for m in &muscles {
                            let mut sel = self.mapping_entry.secondary.contains(m);
                            if ui.checkbox(&mut sel, m).changed() {
                                if sel {
                                    if !self.mapping_entry.secondary.contains(m) {
                                        self.mapping_entry.secondary.push(m.clone());
                                    }
                                } else {
                                    self.mapping_entry.secondary.retain(|s| s != m);
                                }
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Category:");
                            ui.text_edit_singleline(&mut self.mapping_entry.category);
                        });
                        if ui.button("Save Mapping").clicked() {
                            exercise_mapping::set(
                                self.mapping_exercise.clone(),
                                self.mapping_entry.clone(),
                            );
                            self.mapping_dirty = true;
                        }
                        if ui.button("Remove Mapping").clicked() {
                            exercise_mapping::remove(&self.mapping_exercise);
                            self.mapping_dirty = true;
                        }
                    }
                });
            self.show_mapping = open;
        }

        if self.show_settings {
            let prev_start = self.settings.start_date;
            let prev_end = self.settings.end_date;
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::CollapsingHeader::new("Plots")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("plots_grid").num_columns(2).show(ui, |ui| {
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_weight,
                                            "Show Weight over time",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_est_1rm,
                                            "Show Estimated 1RM",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(&mut self.settings.show_sets, "Show Sets per day")
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_rep_histogram,
                                            "Show Rep Histogram",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_volume,
                                            "Show Training Volume",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_body_part_volume,
                                            "Show Volume by Body Part",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_body_part_distribution,
                                            "Show Body Part Distribution",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_exercise_volume,
                                            "Show Exercise Volume",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_weekly_summary,
                                            "Show Weekly Summary",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(&mut self.settings.show_rpe, "Show RPE")
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_rpe_trend,
                                            "Show RPE Trend",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(
                                            &mut self.settings.highlight_max,
                                            "Highlight maximums",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_weight_trend,
                                            "Show Weight Trend",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_smoothed,
                                            "Show moving average",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    if ui
                                        .checkbox(
                                            &mut self.settings.show_volume_trend,
                                            "Show Volume Trend",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();

                                    ui.horizontal(|ui| {
                                        ui.label("MA Window:");
                                        let mut w = self.settings.ma_window.to_string();
                                        if ui.text_edit_singleline(&mut w).changed() {
                                            if let Ok(v) = w.parse::<usize>() {
                                                self.settings.ma_window = v.max(1);
                                                self.settings_dirty = true;
                                            }
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Smoothing:");
                                        let prev = self.settings.smoothing_method;
                                        egui::ComboBox::from_id_source("smoothing_method_combo")
                                            .selected_text(match self.settings.smoothing_method {
                                                SmoothingMethod::SimpleMA => "Simple MA",
                                                SmoothingMethod::EMA => "EMA",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.settings.smoothing_method,
                                                    SmoothingMethod::SimpleMA,
                                                    "Simple MA",
                                                );
                                                ui.selectable_value(
                                                    &mut self.settings.smoothing_method,
                                                    SmoothingMethod::EMA,
                                                    "EMA",
                                                );
                                            });
                                        if prev != self.settings.smoothing_method {
                                            self.settings_dirty = true;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Plot width:");
                                        let mut w = format!("{:.0}", self.settings.plot_width);
                                        if ui.text_edit_singleline(&mut w).changed() {
                                            if let Ok(v) = w.parse::<f32>() {
                                                self.settings.plot_width = v.max(50.0);
                                                self.settings_dirty = true;
                                            }
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Plot height:");
                                        let mut h = format!("{:.0}", self.settings.plot_height);
                                        if ui.text_edit_singleline(&mut h).changed() {
                                            if let Ok(v) = h.parse::<f32>() {
                                                self.settings.plot_height = v.max(50.0);
                                                self.settings_dirty = true;
                                            }
                                        }
                                    });
                                    ui.end_row();
                                });
                            });

                        ui.separator();

                        egui::CollapsingHeader::new("Display")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("display_grid")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Start date:");
                                            let mut start = self
                                                .settings
                                                .start_date
                                                .unwrap_or_else(|| Local::now().date_naive());
                                            if ui
                                                .add(
                                                    DatePickerButton::new(&mut start)
                                                        .id_source("start_date"),
                                                )
                                                .changed()
                                            {
                                                self.settings.start_date = Some(start);
                                                self.settings_dirty = true;
                                            }
                                            if self.settings.start_date.is_some()
                                                && ui.button("Clear").clicked()
                                            {
                                                self.settings.start_date = None;
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("End date:");
                                            let mut end = self
                                                .settings
                                                .end_date
                                                .unwrap_or_else(|| Local::now().date_naive());
                                            if ui
                                                .add(
                                                    DatePickerButton::new(&mut end)
                                                        .id_source("end_date"),
                                                )
                                                .changed()
                                            {
                                                self.settings.end_date = Some(end);
                                                self.settings_dirty = true;
                                            }
                                            if self.settings.end_date.is_some()
                                                && ui.button("Clear").clicked()
                                            {
                                                self.settings.end_date = None;
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("1RM Formula:");
                                            let prev = self.settings.one_rm_formula;
                                            egui::ComboBox::from_id_source("rm_formula_setting")
                                                .selected_text(match self.settings.one_rm_formula {
                                                    OneRmFormula::Epley => "Epley",
                                                    OneRmFormula::Brzycki => "Brzycki",
                                                    OneRmFormula::Lombardi => "Lombardi",
                                                    OneRmFormula::Mayhew => "Mayhew",
                                                    OneRmFormula::OConner => "O'Conner",
                                                    OneRmFormula::Wathan => "Wathan",
                                                    OneRmFormula::Lander => "Lander",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Epley,
                                                        "Epley",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Brzycki,
                                                        "Brzycki",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Lombardi,
                                                        "Lombardi",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Mayhew,
                                                        "Mayhew",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::OConner,
                                                        "O'Conner",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Wathan,
                                                        "Wathan",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.one_rm_formula,
                                                        OneRmFormula::Lander,
                                                        "Lander",
                                                    );
                                                });
                                            if prev != self.settings.one_rm_formula {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("X Axis:");
                                            let prev = self.settings.x_axis;
                                            egui::ComboBox::from_id_source("x_axis_setting")
                                                .selected_text(match self.settings.x_axis {
                                                    XAxis::Date => "Date",
                                                    XAxis::WorkoutIndex => "Workout Index",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.x_axis,
                                                        XAxis::Date,
                                                        "Date",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.x_axis,
                                                        XAxis::WorkoutIndex,
                                                        "Workout Index",
                                                    );
                                                });
                                            if prev != self.settings.x_axis {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Y Axis:");
                                            let prev = self.settings.y_axis;
                                            egui::ComboBox::from_id_source("y_axis_setting")
                                                .selected_text(match self.settings.y_axis {
                                                    YAxis::Weight => "Weight",
                                                    YAxis::Volume => "Volume",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.y_axis,
                                                        YAxis::Weight,
                                                        "Weight",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.y_axis,
                                                        YAxis::Volume,
                                                        "Volume",
                                                    );
                                                });
                                            if prev != self.settings.y_axis {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Weight unit:");
                                            let prev = self.settings.weight_unit;
                                            egui::ComboBox::from_id_source("weight_unit_setting")
                                                .selected_text(match self.settings.weight_unit {
                                                    WeightUnit::Lbs => "lbs",
                                                    WeightUnit::Kg => "kg",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.weight_unit,
                                                        WeightUnit::Lbs,
                                                        "lbs",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.weight_unit,
                                                        WeightUnit::Kg,
                                                        "kg",
                                                    );
                                                });
                                            if prev != self.settings.weight_unit {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Volume agg:");
                                            let prev = self.settings.volume_aggregation;
                                            egui::ComboBox::from_id_source("volume_agg_setting")
                                                .selected_text(
                                                    match self.settings.volume_aggregation {
                                                        VolumeAggregation::Daily => "Daily",
                                                        VolumeAggregation::Weekly => "Weekly",
                                                        VolumeAggregation::Monthly => "Monthly",
                                                    },
                                                )
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.volume_aggregation,
                                                        VolumeAggregation::Daily,
                                                        "Daily",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.volume_aggregation,
                                                        VolumeAggregation::Weekly,
                                                        "Weekly",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self.settings.volume_aggregation,
                                                        VolumeAggregation::Monthly,
                                                        "Monthly",
                                                    );
                                                });
                                            if prev != self.settings.volume_aggregation {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Body part agg:");
                                            let prev = self.settings.body_part_volume_aggregation;
                                            egui::ComboBox::from_id_source(
                                                "body_part_volume_agg_setting",
                                            )
                                            .selected_text(
                                                match self.settings.body_part_volume_aggregation {
                                                    VolumeAggregation::Daily => "Daily",
                                                    VolumeAggregation::Weekly => "Weekly",
                                                    VolumeAggregation::Monthly => "Monthly",
                                                },
                                            )
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut self
                                                            .settings
                                                            .body_part_volume_aggregation,
                                                        VolumeAggregation::Daily,
                                                        "Daily",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self
                                                            .settings
                                                            .body_part_volume_aggregation,
                                                        VolumeAggregation::Weekly,
                                                        "Weekly",
                                                    );
                                                    ui.selectable_value(
                                                        &mut self
                                                            .settings
                                                            .body_part_volume_aggregation,
                                                        VolumeAggregation::Monthly,
                                                        "Monthly",
                                                    );
                                                },
                                            );
                                            if prev != self.settings.body_part_volume_aggregation {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();
                                    });
                            });

                        ui.separator();

                        egui::CollapsingHeader::new("Filtering")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("filter_grid")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Set type filter:");
                                            let prev = self.settings.set_type_filter.clone();
                                            egui::ComboBox::from_id_source("set_type_filter_combo")
                                                .selected_text(prev.as_deref().unwrap_or("All"))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.set_type_filter,
                                                        None::<String>,
                                                        "All",
                                                    );
                                                    for st in &self.set_types {
                                                        ui.selectable_value(
                                                            &mut self.settings.set_type_filter,
                                                            Some(st.clone()),
                                                            st,
                                                        );
                                                    }
                                                });
                                            if prev != self.settings.set_type_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Superset id:");
                                            let prev = self.settings.superset_filter.clone();
                                            egui::ComboBox::from_id_source("superset_filter_combo")
                                                .selected_text(prev.as_deref().unwrap_or("All"))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.superset_filter,
                                                        None::<String>,
                                                        "All",
                                                    );
                                                    for id in &self.superset_ids {
                                                        ui.selectable_value(
                                                            &mut self.settings.superset_filter,
                                                            Some(id.clone()),
                                                            id,
                                                        );
                                                    }
                                                });
                                            if prev != self.settings.superset_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Notes contains:");
                                            let mut nf = self
                                                .settings
                                                .notes_filter
                                                .clone()
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut nf).changed() {
                                                self.settings.notes_filter = if nf.trim().is_empty()
                                                {
                                                    None
                                                } else {
                                                    Some(nf)
                                                };
                                                self.settings_dirty = true;
                                            }
                                        });
                                        if ui
                                            .checkbox(
                                                &mut self.settings.exclude_warmups,
                                                "Exclude warm-up sets",
                                            )
                                            .changed()
                                        {
                                            self.settings_dirty = true;
                                        }
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Body part:");
                                            let prev = self.settings.body_part_filter.clone();
                                            let parts = body_parts::primary_muscle_groups();
                                            egui::ComboBox::from_id_source(
                                                "body_part_filter_combo",
                                            )
                                            .selected_text(prev.as_deref().unwrap_or("All"))
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.body_part_filter,
                                                        None::<String>,
                                                        "All",
                                                    );
                                                    for p in parts {
                                                        ui.selectable_value(
                                                            &mut self.settings.body_part_filter,
                                                            Some(p.clone()),
                                                            &p,
                                                        );
                                                    }
                                                },
                                            );
                                            if prev != self.settings.body_part_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Exercise type:");
                                            let prev = self.settings.exercise_type_filter;
                                            egui::ComboBox::from_id_source(
                                                "exercise_type_filter_combo",
                                            )
                                            .selected_text(match prev {
                                                Some(k) => format!("{:?}", k),
                                                None => "Any".into(),
                                            })
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.exercise_type_filter,
                                                        None::<ExerciseType>,
                                                        "Any",
                                                    );
                                                    for k in body_parts::ALL_EXERCISE_TYPES {
                                                        ui.selectable_value(
                                                            &mut self.settings.exercise_type_filter,
                                                            Some(k),
                                                            format!("{:?}", k),
                                                        );
                                                    }
                                                },
                                            );
                                            if prev != self.settings.exercise_type_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Difficulty:");
                                            let prev = self.settings.difficulty_filter;
                                            egui::ComboBox::from_id_source(
                                                "difficulty_filter_combo",
                                            )
                                            .selected_text(match prev {
                                                Some(d) => format!("{:?}", d),
                                                None => "Any".into(),
                                            })
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.difficulty_filter,
                                                        None::<body_parts::Difficulty>,
                                                        "Any",
                                                    );
                                                    for d in body_parts::ALL_DIFFICULTIES {
                                                        ui.selectable_value(
                                                            &mut self.settings.difficulty_filter,
                                                            Some(d),
                                                            format!("{:?}", d),
                                                        );
                                                    }
                                                },
                                            );
                                            if prev != self.settings.difficulty_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Equipment:");
                                            let prev = self.settings.equipment_filter;
                                            egui::ComboBox::from_id_source(
                                                "equipment_filter_combo",
                                            )
                                            .selected_text(match prev {
                                                Some(e) => format!("{:?}", e),
                                                None => "Any".into(),
                                            })
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut self.settings.equipment_filter,
                                                        None::<body_parts::Equipment>,
                                                        "Any",
                                                    );
                                                    for e in body_parts::ALL_EQUIPMENT {
                                                        ui.selectable_value(
                                                            &mut self.settings.equipment_filter,
                                                            Some(e),
                                                            format!("{:?}", e),
                                                        );
                                                    }
                                                },
                                            );
                                            if prev != self.settings.equipment_filter {
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Min RPE:");
                                            let mut min = self
                                                .settings
                                                .min_rpe
                                                .map(|v| format!("{:.1}", v))
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut min).changed() {
                                                self.settings.min_rpe = min.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                            ui.label("Max RPE:");
                                            let mut max = self
                                                .settings
                                                .max_rpe
                                                .map(|v| format!("{:.1}", v))
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut max).changed() {
                                                self.settings.max_rpe = max.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Min weight:");
                                            let mut mw = self
                                                .settings
                                                .min_weight
                                                .map(|v| format!("{:.1}", v))
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut mw).changed() {
                                                self.settings.min_weight = mw.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                            ui.label("Max weight:");
                                            let mut mxw = self
                                                .settings
                                                .max_weight
                                                .map(|v| format!("{:.1}", v))
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut mxw).changed() {
                                                self.settings.max_weight = mxw.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();

                                        ui.horizontal(|ui| {
                                            ui.label("Min reps:");
                                            let mut mr = self
                                                .settings
                                                .min_reps
                                                .map(|v| v.to_string())
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut mr).changed() {
                                                self.settings.min_reps = mr.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                            ui.label("Max reps:");
                                            let mut mxr = self
                                                .settings
                                                .max_reps
                                                .map(|v| v.to_string())
                                                .unwrap_or_default();
                                            if ui.text_edit_singleline(&mut mxr).changed() {
                                                self.settings.max_reps = mxr.trim().parse().ok();
                                                self.settings_dirty = true;
                                            }
                                        });
                                        ui.end_row();
                                    });
                            });

                        ui.separator();

                        egui::CollapsingHeader::new("Data Import")
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("data_grid").num_columns(2).show(ui, |ui| {
                                    if ui
                                        .checkbox(
                                            &mut self.settings.auto_load_last,
                                            "Auto-load last file",
                                        )
                                        .changed()
                                    {
                                        self.settings_dirty = true;
                                    }
                                    ui.end_row();
                                });
                            });
                    });
                });

            if (self.settings.start_date != prev_start || self.settings.end_date != prev_end)
                && !self.workouts.is_empty()
            {
                self.stats = compute_stats(
                    &self.workouts,
                    self.settings.start_date,
                    self.settings.end_date,
                );
            }
        }

        if let Some(start) = self.toast_start {
            if start.elapsed() < Duration::from_secs(3) {
                let file = self.last_loaded.as_deref().unwrap_or("file");
                egui::Area::new(egui::Id::new("load_toast"))
                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                    .show(ctx, |ui| {
                        ui.label(format_load_message(self.workouts.len(), file));
                    });
            } else {
                self.toast_start = None;
            }
        }

        if let Some(start) = self.pr_toast_start {
            if start.elapsed() < Duration::from_secs(3) {
                if let Some(ref msg) = self.pr_message {
                    egui::Area::new(egui::Id::new("pr_toast"))
                        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
                        .show(ctx, |ui| {
                            ui.label(msg);
                        });
                }
            } else {
                self.pr_toast_start = None;
                self.pr_message = None;
            }
        }

        if self.settings_dirty {
            self.settings.save();
            self.settings_dirty = false;
        }
        if self.mapping_dirty {
            exercise_mapping::save();
            self.mapping_dirty = false;
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.sync_settings_from_app();
        self.settings.save();
        exercise_mapping::save();
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = NativeOptions::default();
    eframe::run_native(
        "Multi Hevy Workout Dashboard",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn settings_roundtrip() {
        let mut s = Settings::default();
        s.show_weight = false;
        s.show_est_1rm = false;
        s.show_sets = false;
        s.show_rpe = true;
        s.show_rpe_trend = true;
        s.show_weight_trend = true;
        s.show_volume_trend = true;
        s.show_smoothed = true;
        s.ma_window = 3;
        s.smoothing_method = SmoothingMethod::EMA;
        s.one_rm_formula = OneRmFormula::Brzycki;
        s.start_date = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        s.end_date = Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        s.x_axis = XAxis::WorkoutIndex;
        s.y_axis = YAxis::Volume;
        s.weight_unit = WeightUnit::Kg;
        s.set_type_filter = Some("working".into());
        s.superset_filter = Some("A".into());
        s.body_part_filter = Some("Chest".into());
        s.exercise_type_filter = Some(ExerciseType::Compound);
        s.min_rpe = Some(6.0);
        s.max_rpe = Some(9.0);
        s.min_weight = Some(135.0);
        s.max_weight = Some(225.0);
        s.min_reps = Some(3);
        s.max_reps = Some(10);
        s.notes_filter = Some("tempo".into());
        s.exclude_warmups = true;
        s.show_body_part_volume = true;
        s.show_body_part_distribution = true;
        s.show_exercise_volume = true;
        s.show_weekly_summary = true;
        s.show_exercise_stats = true;
        s.show_personal_records = true;
        s.show_exercise_panel = false;
        s.body_part_volume_aggregation = VolumeAggregation::Monthly;
        s.auto_load_last = false;
        s.last_file = Some("/tmp/test.csv".into());
        s.check_prs = true;
        s.github_repo = Some("user/repo".into());
        s.last_pr = Some(5);
        s.selected_exercises = vec!["Bench".into()];
        s.table_filter = "bench".into();
        s.sort_column = SortColumn::Weight;
        s.sort_ascending = false;
        s.summary_sort = SummarySort::Volume;
        s.summary_sort_ascending = false;

        let json = serde_json::to_string(&s).unwrap();
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn show_rpe_persistence() {
        use std::env;
        use std::fs;

        let _guard = ENV_MUTEX.lock().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let prev_config = env::var_os("XDG_CONFIG_HOME");
        unsafe {
            env::set_var("XDG_CONFIG_HOME", dir.path());
        }

        let mut s = Settings::default();
        s.show_rpe = true;
        s.save();
        let loaded = Settings::load();
        assert!(loaded.show_rpe);

        let path = Settings::path().unwrap();
        fs::write(&path, "{}").unwrap();
        let missing = Settings::load();
        assert!(!missing.show_rpe);

        if let Some(val) = prev_config {
            unsafe {
                env::set_var("XDG_CONFIG_HOME", val);
            }
        } else {
            unsafe {
                env::remove_var("XDG_CONFIG_HOME");
            }
        }
    }

    #[test]
    fn show_rpe_ui_toggle_persists() {
        use std::env;

        let _guard = ENV_MUTEX.lock().unwrap();

        // Use a temporary config directory so the test does not affect real files.
        let dir = tempfile::tempdir().unwrap();
        let prev_config = env::var_os("XDG_CONFIG_HOME");
        unsafe {
            env::set_var("XDG_CONFIG_HOME", dir.path());
        }

        let mut app = MyApp::default();
        app.settings.show_rpe = false;

        let ctx = egui::Context::default();

        // Open the settings window; the checkbox is present but remains false.
        let _ = ctx.run(Default::default(), |ctx| {
            egui::Window::new("Settings").show(ctx, |ui| {
                if ui
                    .checkbox(&mut app.settings.show_rpe, "Show RPE")
                    .changed()
                {
                    app.settings_dirty = true;
                }
            });
        });

        // Simulate the user toggling the checkbox.
        app.settings.show_rpe = true;
        app.settings_dirty = true;

        assert!(app.settings.show_rpe);
        assert!(app.settings_dirty);
        app.settings.save();
        let loaded = Settings::load();
        assert!(loaded.show_rpe);

        if let Some(val) = prev_config {
            unsafe {
                env::set_var("XDG_CONFIG_HOME", val);
            }
        } else {
            unsafe {
                env::remove_var("XDG_CONFIG_HOME");
            }
        }
    }

    #[test]
    fn parse_workout_csv_basic() {
        let data = "title,start_time,end_time,description,exercise_title,superset_id,exercise_notes,set_index,set_type,weight_lbs,reps,distance_miles,duration_seconds,rpe\n\
Week 12 - Lower - Strength,\"26 Jul 2025, 07:06\",\"26 Jul 2025, 08:11\",desc,\"Lying Leg Curl (Machine)\",,,0,warmup,100,10,,,\n";
        let entries = parse_workout_csv(data.as_bytes()).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.date, "2025-07-26");
        assert_eq!(e.exercise, "Lying Leg Curl (Machine)");
        assert_eq!(e.weight, 100.0);
        assert_eq!(e.reps, 10);
        assert_eq!(e.raw.exercise_title, "Lying Leg Curl (Machine)");
        assert_eq!(e.raw.weight_lbs, Some(100.0));
        assert_eq!(e.raw.reps, Some(10));
    }

    #[test]
    fn parse_workout_csv_weight_kg() {
        let data = "title,start_time,end_time,description,exercise_title,superset_id,exercise_notes,set_index,set_type,weight_kg,reps,distance_miles,duration_seconds,rpe\n\
Week 1 - Upper,\"27 Jul 2025, 07:00\",,desc,Bench Press,,,0,working,50,8,,,\n";
        let entries = parse_workout_csv(data.as_bytes()).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.date, "2025-07-27");
        assert_eq!(e.exercise, "Bench Press");
        assert!((e.weight - 110.231).abs() < 0.01);
        assert_eq!(e.reps, 8);
        assert_eq!(e.raw.weight_lbs, None);
        assert_eq!(e.raw.weight_kg, Some(50.0));
    }

    #[test]
    fn body_part_filter() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Squat".into(),
                weight: 150.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.body_part_filter = Some("Chest".into());
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].exercise, "Bench");
    }

    #[test]
    fn exercise_type_filter() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Lying Leg Curl (Machine)".into(),
                weight: 100.0,
                reps: 10,
                raw: RawWorkoutRow::default(),
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.exercise_type_filter = Some(ExerciseType::Isolation);
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].exercise, "Lying Leg Curl (Machine)");
    }

    #[test]
    fn difficulty_filter() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Push-Up".into(),
                weight: 0.0,
                reps: 15,
                raw: RawWorkoutRow::default(),
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.difficulty_filter = Some(body_parts::Difficulty::Beginner);
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].exercise, "Push-Up");
    }

    #[test]
    fn equipment_filter() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Lying Leg Curl (Machine)".into(),
                weight: 100.0,
                reps: 10,
                raw: RawWorkoutRow::default(),
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.equipment_filter = Some(body_parts::Equipment::Machine);
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].exercise, "Lying Leg Curl (Machine)");
    }

    #[test]
    fn exclude_warmups() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 45.0,
                reps: 10,
                raw: RawWorkoutRow {
                    set_type: Some("warmup".into()),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow {
                    set_type: Some("working".into()),
                    ..RawWorkoutRow::default()
                },
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.exclude_warmups = true;
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].weight, 100.0);
    }

    #[test]
    fn weight_and_rep_filters() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow::default(),
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Bench".into(),
                weight: 200.0,
                reps: 10,
                raw: RawWorkoutRow::default(),
            },
        ];
        let mut app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        app.settings.min_weight = Some(150.0);
        app.settings.max_weight = Some(250.0);
        app.settings.min_reps = Some(8);
        app.settings.max_reps = Some(12);
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].weight, 200.0);
        assert_eq!(filtered[0].reps, 10);
    }

    #[test]
    fn exercise_set_counts() {
        let entries = vec![
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 45.0,
                reps: 10,
                raw: RawWorkoutRow {
                    set_type: Some("warmup".into()),
                    title: Some("W1".into()),
                    start_time: "1".into(),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-01".into(),
                exercise: "Bench".into(),
                weight: 100.0,
                reps: 5,
                raw: RawWorkoutRow {
                    set_type: Some("working".into()),
                    title: Some("W1".into()),
                    start_time: "1".into(),
                    ..RawWorkoutRow::default()
                },
            },
            WorkoutEntry {
                date: "2024-01-02".into(),
                exercise: "Bench".into(),
                weight: 105.0,
                reps: 5,
                raw: RawWorkoutRow {
                    set_type: Some("working".into()),
                    title: Some("W2".into()),
                    start_time: "2".into(),
                    ..RawWorkoutRow::default()
                },
            },
        ];
        let app = MyApp {
            workouts: entries,
            ..Default::default()
        };
        let (w, working, warmup) = app.exercise_set_counts("Bench");
        assert_eq!(w, 2);
        assert_eq!(working, 2);
        assert_eq!(warmup, 1);
    }

    #[test]
    fn sort_summary_stats_by_column() {
        let mut stats = vec![
            (
                "Bench".to_string(),
                ExerciseStats {
                    total_sets: 2,
                    total_reps: 10,
                    total_volume: 200.0,
                    best_est_1rm: Some(150.0),
                    max_weight: Some(120.0),
                    ..Default::default()
                },
            ),
            (
                "Squat".to_string(),
                ExerciseStats {
                    total_sets: 1,
                    total_reps: 5,
                    total_volume: 300.0,
                    best_est_1rm: Some(250.0),
                    max_weight: Some(200.0),
                    ..Default::default()
                },
            ),
            (
                "Deadlift".to_string(),
                ExerciseStats {
                    total_sets: 3,
                    total_reps: 15,
                    total_volume: 400.0,
                    best_est_1rm: Some(350.0),
                    max_weight: Some(300.0),
                    ..Default::default()
                },
            ),
        ];

        MyApp::sort_summary_stats(&mut stats, SummarySort::Exercise, true);
        assert_eq!(stats[0].0, "Bench");
        assert_eq!(stats[1].0, "Deadlift");
        assert_eq!(stats[2].0, "Squat");

        MyApp::sort_summary_stats(&mut stats, SummarySort::Reps, false);
        assert_eq!(stats[0].0, "Deadlift");
        assert_eq!(stats[1].0, "Bench");
        assert_eq!(stats[2].0, "Squat");

        MyApp::sort_summary_stats(&mut stats, SummarySort::MaxWeight, false);
        assert_eq!(stats[0].0, "Deadlift");
        assert_eq!(stats[1].0, "Squat");
        assert_eq!(stats[2].0, "Bench");
    }

    #[test]
    fn test_parse_latest_pr_number() {
        let json = "[{\"number\": 42}]";
        assert_eq!(parse_latest_pr_number(json), Some(42));
    }
}
