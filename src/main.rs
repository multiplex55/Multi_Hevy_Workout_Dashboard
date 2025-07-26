use eframe::{App, Frame, NativeOptions, egui};
use egui_plot::Plot;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs::File;

mod analysis;
use analysis::{BasicStats, compute_stats};
mod plotting;
use plotting::{estimated_1rm_line, sets_per_day_bar, unique_exercises, weight_over_time_line};

#[derive(Debug, Deserialize, Clone)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
    stats: BasicStats,
    selected_exercise: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
            stats: BasicStats::default(),
            selected_exercise: None,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load CSV").clicked() {
                if let Some(path) = FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                    if let Ok(file) = File::open(path) {
                        let mut rdr = csv::Reader::from_reader(file);
                        self.workouts = rdr.deserialize().filter_map(|res| res.ok()).collect();
                        self.stats = compute_stats(&self.workouts);
                        if self.selected_exercise.is_none() {
                            self.selected_exercise =
                                unique_exercises(&self.workouts).into_iter().next();
                        }
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
                        plot_ui.line(weight_over_time_line(&self.workouts, ex));
                        plot_ui.line(estimated_1rm_line(&self.workouts, ex));
                        plot_ui.bar_chart(sets_per_day_bar(&self.workouts, Some(ex)));
                    });
                }
            }

            ui.heading("Loaded Workouts");
            for entry in &self.workouts {
                ui.label(format!("{:?}", entry));
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "Multi Hevy Workout Dashboard",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
