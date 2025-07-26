use eframe::{egui, App, Frame, NativeOptions};
use serde::Deserialize;
use rfd::FileDialog;
use std::fs::File;

mod analysis;
use analysis::{compute_stats, BasicStats};

#[derive(Debug, Deserialize)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
    stats: BasicStats,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
            stats: BasicStats::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load CSV").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .pick_file()
                {
                    if let Ok(file) = File::open(path) {
                        let mut rdr = csv::Reader::from_reader(file);
                        self.workouts = rdr
                            .deserialize()
                            .filter_map(|res| res.ok())
                            .collect();
                        self.stats = compute_stats(&self.workouts);
                    }
                }
            }

            if !self.workouts.is_empty() {
                ui.heading("Workout Statistics");
                ui.label(format!("Total workouts: {}", self.stats.total_workouts));
                ui.label(format!("Avg sets/workout: {:.2}", self.stats.avg_sets_per_workout));
                ui.label(format!("Avg reps/set: {:.2}", self.stats.avg_reps_per_set));
                ui.label(format!("Avg days between: {:.2}", self.stats.avg_days_between));
                if let Some(ref ex) = self.stats.most_common_exercise {
                    ui.label(format!("Most common exercise: {}", ex));
                }
                ui.separator();
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
        "Minimal egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
