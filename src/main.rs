use eframe::{egui, App, Frame, NativeOptions};
use serde::Deserialize;
use rfd::FileDialog;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct WorkoutEntry {
    date: String,
    exercise: String,
    weight: f32,
    reps: u32,
}

struct MyApp {
    workouts: Vec<WorkoutEntry>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            workouts: Vec::new(),
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
                    }
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
        "Minimal egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
