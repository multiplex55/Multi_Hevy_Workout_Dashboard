use eframe::{App, Frame, NativeOptions, egui};
use egui_plot::Plot;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs::File;
use std::time::{Duration, Instant};

use log::info;

mod analysis;
use analysis::{BasicStats, compute_stats, format_load_message};
mod plotting;
use plotting::{
    OneRmFormula, estimated_1rm_line, sets_per_day_bar, unique_exercises, weight_over_time_line,
};

#[derive(Debug, Deserialize, Clone)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
}

#[derive(Clone)]
struct Settings {
    show_weight: bool,
    show_est_1rm: bool,
    show_sets: bool,
    one_rm_formula: OneRmFormula,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_weight: true,
            show_est_1rm: true,
            show_sets: true,
            one_rm_formula: OneRmFormula::Epley,
        }
    }
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
    stats: BasicStats,
    selected_exercise: Option<String>,
    last_loaded: Option<String>,
    toast_start: Option<Instant>,
    settings: Settings,
    show_settings: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
            stats: BasicStats::default(),
            selected_exercise: None,
            last_loaded: None,
            toast_start: None,
            settings: Settings::default(),
            show_settings: false,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                });
            });
        });
        egui::SidePanel::left("info_panel").show(ctx, |ui| {
            if self.workouts.is_empty() {
                ui.label("No CSV loaded");
                ui.label("Load a CSV to begin");
            } else {
                ui.label(format!("Loaded {} entries", self.workouts.len()));
            }

            ui.separator();
            if let Some(ref ex) = self.selected_exercise {
                ui.label(format!("Selected exercise: {}", ex));
            } else {
                ui.label("No exercise selected");
                ui.label("Select an exercise from the dropdown");
            }

            ui.separator();
            ui.collapsing("Available plots", |ui| {
                ui.label("• Weight over time");
                ui.label("• Estimated 1RM");
                ui.label("• Sets per day");
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
                        let mut rdr = csv::Reader::from_reader(file);
                        self.workouts = rdr.deserialize().filter_map(|res| res.ok()).collect();
                        info!("Loaded {} entries from {}", self.workouts.len(), filename);
                        self.stats = compute_stats(&self.workouts);
                        if self.selected_exercise.is_none() {
                            self.selected_exercise =
                                unique_exercises(&self.workouts).into_iter().next();
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

                let exercises = unique_exercises(&self.workouts);
                if self.selected_exercise.is_none() {
                    self.selected_exercise = exercises.first().cloned();
                }
                ui.horizontal(|ui| {
                    ui.label("Exercise:");
                    egui::ComboBox::from_id_source("exercise_combo")
                        .selected_text(
                            self.selected_exercise
                                .as_ref()
                                .cloned()
                                .unwrap_or_else(|| "".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for ex in &exercises {
                                ui.selectable_value(
                                    &mut self.selected_exercise,
                                    Some(ex.clone()),
                                    ex,
                                );
                            }
                        });
                });

                if let Some(ref ex) = self.selected_exercise {
                    Plot::new("exercise_plot").show(ui, |plot_ui| {
                        if self.settings.show_weight {
                            plot_ui.line(weight_over_time_line(&self.workouts, ex));
                        }
                        if self.settings.show_est_1rm {
                            plot_ui.line(estimated_1rm_line(
                                &self.workouts,
                                ex,
                                self.settings.one_rm_formula,
                            ));
                        }
                        if self.settings.show_sets {
                            plot_ui.bar_chart(sets_per_day_bar(&self.workouts, Some(ex)));
                        }
                    });
                }
            }

            ui.heading("Loaded Workouts");
            for entry in &self.workouts {
                ui.label(format!("{:?}", entry));
            }
        });

        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.settings.show_weight, "Show Weight over time");
                    ui.checkbox(&mut self.settings.show_est_1rm, "Show Estimated 1RM");
                    ui.checkbox(&mut self.settings.show_sets, "Show Sets per day");
                    ui.horizontal(|ui| {
                        ui.label("1RM Formula:");
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
