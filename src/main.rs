use eframe::{egui, App, Frame, NativeOptions};

struct MyApp;

impl Default for MyApp {
    fn default() -> Self {
        Self
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello from egui!");
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
