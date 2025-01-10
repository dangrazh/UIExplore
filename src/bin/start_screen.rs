#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use uiexplore::signal_file;

fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([420.0, 240.0])
        .with_decorations(false)
        .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "UI Explore",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::new()))
        }),
    )
}

struct MyApp {}

impl MyApp {
    fn new() -> Self {  
        Self {}
    }
}

impl eframe::App for MyApp {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::center_on_screen(ctx).unwrap());

        let layout = egui::Layout::top_down(egui::Align::Center);

        egui::CentralPanel::default().show(ctx, |ui| {            
            ui.with_layout(layout, |ui| {                
                ui.add_space(70.0);
                ui.heading("UI Explore");
                ui.add_space(20.0);
                ui.label("Please wait while we prepare the data...");
            });
        });

        
        // check if parent process is done and if yes, close the app
        if signal_file::termination_signal() {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
        }

    }
}