#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// #![allow(warnings)]

#[macro_use]
mod macros;

use windows::Win32::UI::WindowsAndMessaging::*;

mod rectangle;

use ::uiexplore::signal_file;

mod uiexplore;
use uiexplore::{UITree, UIElementProps};

mod app_ui;
use app_ui::UIExplorer;

use eframe::egui;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

pub type UIHashMap<K, V, S = std::hash::RandomState> = std::collections::HashMap<K, V, S>;
type UIHashSet<T, S = std::hash::RandomState> = std::collections::HashSet<T, S>;

mod tree_map;
use tree_map::UITreeMap;


fn main() -> eframe::Result {

    printfmt!("Getting the ui tree");

    // get the ui tree in a separate thread
    let (tx, rx): (Sender<_>, Receiver<uiexplore::UITree>) = channel();
    thread::spawn(|| {
        uiexplore::get_all_elements(tx, None);
    });
    printfmt!("Spawned separate thread to get ui tree");

    printfmt!("displaying start screen now");
    launch_start_screen();
    
    let ui_tree = rx.recv().unwrap();
    
    signal_file::create_signal_file().unwrap();
    printfmt!("UI Tree retrieved, setting up UIExplorer app...");

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).    

    let app_size_pos = AppSizeAndPosition::new_from_screen(0.6);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
                    .with_inner_size([app_size_pos.app_width as f32, app_size_pos.app_height as f32])
                    .with_position(egui::Pos2::new(app_size_pos.app_left as f32, app_size_pos.app_top as f32))
                    .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "UI Explorer",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(UIExplorer::new_with_state(ui_tree)))
        }),

    )
}

#[repr(C)]
struct ScreenSize {
    width: i32,
    height: i32,
}

#[derive(Debug)]
#[repr(C)]
struct AppSizeAndPosition {
    screen_width: i32,
    screen_height: i32,
    app_width: f32,
    app_height: f32,
    app_left: f32,
    app_top: f32,
}

impl AppSizeAndPosition {
    fn new(screen_width: i32, screen_height: i32, app_width: f32, app_height: f32, app_left: f32, app_top: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            app_width,
            app_height,
            app_left,
            app_top,
        }
    }

    fn new_from_screen(scaling: f32) -> Self {
        let screen_size = get_system_metrics();
        let screen_width = screen_size.width;
        let screen_height = screen_size.height;
        let app_width = screen_width as f32 * scaling;
        let app_height = screen_height as f32 * scaling;
        let app_left = screen_width as f32 / 2.0 - app_width / 2.0;
        let app_top = screen_height as f32 / 2.0 - app_height / 2.0;
        Self::new(screen_width, screen_height, app_width, app_height, app_left, app_top)
    }
}

extern "system" fn get_system_metrics() -> ScreenSize {
    unsafe {
        let x = GetSystemMetrics(SM_CXSCREEN);
        let y = GetSystemMetrics(SM_CYSCREEN);
        // println!("Screen size: {}x{}", x, y);
        ScreenSize { width: x, height: y }
    }
}



fn launch_start_screen() {

    let _cmd = std::process::Command::new("start_screen.exe")
        .spawn()
        .expect("Failed to start start_screen.exe");
}

