use dirs_next as dirs;
use eframe::{App, Frame, NativeOptions, egui};
use egui_extras::DatePickerButton;
use egui_plot::{Plot, Points, MarkerShape};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::time::{Duration, Instant};

use chrono::{Local, NaiveDate};
use log::info;

mod analysis;
use analysis::{BasicStats, compute_stats, format_load_message};
mod plotting;
use plotting::{
    OneRmFormula, XAxis, YAxis, estimated_1rm_line, sets_per_day_bar,
    training_volume_line, unique_exercises, weight_over_time_line,
};
mod capture;
use capture::{crop_image, save_png};
mod export;
use export::{save_entries_csv, save_entries_json, save_stats_csv, save_stats_json};

#[derive(Debug, Deserialize, Clone, Serialize)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
    raw: RawWorkoutRow,
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
                let weight = raw.weight_lbs.unwrap_or(0.0);
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Settings {
    show_weight: bool,
    show_est_1rm: bool,
    show_sets: bool,
    show_volume: bool,
    highlight_max: bool,
    weight_unit: WeightUnit,
    one_rm_formula: OneRmFormula,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    x_axis: XAxis,
    y_axis: YAxis,
}

impl Settings {
    const FILE: &'static str = "multi_hevy_settings.json";

    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join(Self::FILE))
    }

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
    fn default() -> Self {
        Self {
            show_weight: true,
            show_est_1rm: true,
            show_sets: true,
            show_volume: false,
            highlight_max: true,
            weight_unit: WeightUnit::Lbs,
            one_rm_formula: OneRmFormula::Epley,
            start_date: None,
            end_date: None,
            x_axis: XAxis::Date,
            y_axis: YAxis::Weight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Date,
    Exercise,
    Weight,
    Reps,
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
    stats: BasicStats,
    selected_exercises: Vec<String>,
    search_query: String,
    table_filter: String,
    last_loaded: Option<String>,
    toast_start: Option<Instant>,
    settings: Settings,
    show_settings: bool,
    show_entries: bool,
    sort_column: SortColumn,
    sort_ascending: bool,
    capture_rect: Option<egui::Rect>,
    settings_dirty: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
            stats: BasicStats::default(),
            selected_exercises: Vec::new(),
            search_query: String::new(),
            table_filter: String::new(),
            last_loaded: None,
            toast_start: None,
            settings: Settings::load(),
            show_settings: false,
            show_entries: false,
            sort_column: SortColumn::Date,
            sort_ascending: true,
            capture_rect: None,
            settings_dirty: false,
        }
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
                    if ui.button("Export Stats").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .add_filter("CSV", &["csv"])
                            .save_file()
                        {
                            let exercises = analysis::aggregate_exercise_stats(
                                &self.workouts,
                                self.settings.one_rm_formula,
                                self.settings.start_date,
                                self.settings.end_date,
                            )
                            .into_iter()
                            .collect::<Vec<_>>();
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
                    ui.label(format!(
                        "Selected: {}",
                        self.selected_exercises.join(", ")
                    ));
                }

                ui.separator();
                ui.collapsing("Available plots", |ui| {
                    ui.label("• Weight over time");
                    ui.label("• Estimated 1RM");
                    ui.label("• Sets per day");
                    ui.label("• Training volume");
                });
            });
        });

        egui::SidePanel::right("exercise_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if !self.workouts.is_empty() {
                    ui.heading("Exercise Summary");
                    let mut stats = analysis::aggregate_exercise_stats(
                        &self.workouts,
                        self.settings.one_rm_formula,
                        self.settings.start_date,
                        self.settings.end_date,
                    )
                    .into_iter()
                    .collect::<Vec<_>>();
                    stats.sort_by_key(|(ex, _)| ex.clone());
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("exercise_summary_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Exercise");
                                ui.label("Sets");
                                ui.label("Reps");
                                ui.label("Volume");
                                ui.label("Best 1RM");
                                ui.end_row();
                                for (ex, s) in stats {
                                    ui.label(ex);
                                    ui.label(s.total_sets.to_string());
                                    ui.label(s.total_reps.to_string());
                                    let f = self.settings.weight_unit.factor();
                                    ui.label(format!("{:.1}", s.total_volume * f));
                                    if let Some(b) = s.best_est_1rm {
                                        ui.label(format!("{:.1}", b * f));
                                    } else {
                                        ui.label("-");
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
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
                        if self.selected_exercises.is_empty() {
                            if let Some(first) = unique_exercises(
                                &self.workouts,
                                self.settings.start_date,
                                self.settings.end_date,
                            )
                            .into_iter()
                            .next()
                            {
                                self.selected_exercises.push(first);
                            }
                        }
                        self.last_loaded = Some(filename);
                        self.toast_start = Some(Instant::now());
                    }
                }
            }

            if !self.workouts.is_empty() {
                ui.heading("Workout Statistics");
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
                ui.separator();

                let mut exercises = unique_exercises(
                    &self.workouts,
                    self.settings.start_date,
                    self.settings.end_date,
                );
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    exercises.retain(|e| e.to_lowercase().contains(&q));
                }
                if self.selected_exercises.is_empty() {
                    if let Some(first) = exercises.first().cloned() {
                        self.selected_exercises.push(first);
                    }
                }
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.search_query);
                });
                ui.horizontal(|ui| {
                    ui.label("Exercises:");
                    egui::ComboBox::from_id_source("exercise_combo")
                        .selected_text(
                            if self.selected_exercises.is_empty() {
                                String::new()
                            } else {
                                self.selected_exercises.join(", ")
                            },
                        )
                        .show_ui(ui, |ui| {
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
                                }
                            }
                        });
                });

                if !self.selected_exercises.is_empty() {
                    let sel: Vec<String> = self.selected_exercises.clone();
                    let plot_resp = Plot::new("exercise_plot").show(ui, |plot_ui| {
                        if self.settings.show_weight {
                            for lw in weight_over_time_line(
                                &self.workouts,
                                &sel,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.y_axis,
                                self.settings.weight_unit,
                            ) {
                                plot_ui.line(lw.line);
                                if self.settings.highlight_max {
                                    if let Some(p) = lw.max_point {
                                        plot_ui.points(
                                            Points::new(vec![p])
                                                .shape(MarkerShape::Diamond)
                                                .color(egui::Color32::RED)
                                                .name("Max Weight"),
                                        );
                                    }
                                }
                            }
                        }
                        if self.settings.show_est_1rm {
                            for lr in estimated_1rm_line(
                                &self.workouts,
                                &sel,
                                self.settings.one_rm_formula,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.weight_unit,
                            ) {
                                plot_ui.line(lr.line);
                                if self.settings.highlight_max {
                                    if let Some(p) = lr.max_point {
                                        plot_ui.points(
                                            Points::new(vec![p])
                                                .shape(MarkerShape::Circle)
                                                .color(egui::Color32::BLUE)
                                                .name("Max 1RM"),
                                        );
                                    }
                                }
                            }
                        }
                        if self.settings.show_volume {
                            plot_ui.line(training_volume_line(
                                &self.workouts,
                                self.settings.start_date,
                                self.settings.end_date,
                                self.settings.x_axis,
                                self.settings.y_axis,
                                self.settings.weight_unit,
                            ));
                        }
                        if self.settings.show_sets {
                            let ex_for_sets = if sel.len() == 1 {
                                Some(sel[0].as_str())
                            } else {
                                None
                            };
                            plot_ui.bar_chart(sets_per_day_bar(
                                &self.workouts,
                                ex_for_sets,
                                self.settings.start_date,
                                self.settings.end_date,
                            ));
                        }
                    });
                    if ui.button("Save Plot").clicked() {
                        self.capture_rect = Some(plot_resp.response.rect);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                    }
                }
            }

            ui.heading("Workout Entries");
            if ui.button("Open Table").clicked() {
                self.show_entries = true;
            }
        });

        if self.show_entries {
            egui::Window::new("Workout Entries")
                .open(&mut self.show_entries)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filter exercise:");
                        ui.text_edit_singleline(&mut self.table_filter);
                    });
                    let mut entries: Vec<&WorkoutEntry> = self.workouts.iter().collect();
                    if !self.table_filter.is_empty() {
                        let q = self.table_filter.to_lowercase();
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
                    entries.sort_by(|a, b| match self.sort_column {
                        SortColumn::Date => a.date.cmp(&b.date),
                        SortColumn::Exercise => a.exercise.cmp(&b.exercise),
                        SortColumn::Weight => a
                            .weight
                            .partial_cmp(&b.weight)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        SortColumn::Reps => a.reps.cmp(&b.reps),
                    });
                    if !self.sort_ascending {
                        entries.reverse();
                    }
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);
                    let mut sort_column = self.sort_column;
                    let mut sort_ascending = self.sort_ascending;
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
                    self.sort_column = sort_column;
                    self.sort_ascending = sort_ascending;
                });
        }

        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    if ui
                        .checkbox(&mut self.settings.show_weight, "Show Weight over time")
                        .changed()
                    {
                        self.settings_dirty = true;
                    }
                    if ui
                        .checkbox(&mut self.settings.show_est_1rm, "Show Estimated 1RM")
                        .changed()
                    {
                        self.settings_dirty = true;
                    }
                    if ui
                        .checkbox(&mut self.settings.show_sets, "Show Sets per day")
                        .changed()
                    {
                        self.settings_dirty = true;
                    }
                    if ui
                        .checkbox(&mut self.settings.show_volume, "Show Training Volume")
                        .changed()
                    {
                        self.settings_dirty = true;
                    }
                    if ui
                        .checkbox(&mut self.settings.highlight_max, "Highlight maximums")
                        .changed()
                    {
                        self.settings_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Start date:");
                        let mut start = self
                            .settings
                            .start_date
                            .unwrap_or_else(|| Local::now().date_naive());
                        if ui
                            .add(DatePickerButton::new(&mut start).id_source("start_date"))
                            .changed()
                        {
                            self.settings.start_date = Some(start);
                            self.settings_dirty = true;
                        }
                        if self.settings.start_date.is_some() && ui.button("Clear").clicked() {
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
                            .add(DatePickerButton::new(&mut end).id_source("end_date"))
                            .changed()
                        {
                            self.settings.end_date = Some(end);
                            self.settings_dirty = true;
                        }
                        if self.settings.end_date.is_some() && ui.button("Clear").clicked() {
                            self.settings.end_date = None;
                            self.settings_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("1RM Formula:");
                        let prev = self.settings.one_rm_formula;
                        egui::ComboBox::from_id_source("rm_formula_setting")
                            .selected_text(match self.settings.one_rm_formula {
                                OneRmFormula::Epley => "Epley",
                                OneRmFormula::Brzycki => "Brzycki",
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
                                ui.selectable_value(&mut self.settings.x_axis, XAxis::Date, "Date");
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
                });
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

        if self.settings_dirty {
            self.settings.save();
            self.settings_dirty = false;
        }
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

    #[test]
    fn settings_roundtrip() {
        let mut s = Settings::default();
        s.show_weight = false;
        s.show_est_1rm = false;
        s.show_sets = false;
        s.one_rm_formula = OneRmFormula::Brzycki;
        s.start_date = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        s.end_date = Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        s.x_axis = XAxis::WorkoutIndex;
        s.y_axis = YAxis::Volume;
        s.weight_unit = WeightUnit::Kg;

        let json = serde_json::to_string(&s).unwrap();
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
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
}
