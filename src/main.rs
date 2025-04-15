#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// #![allow(warnings)]

#[macro_use]
mod macros;

// use windows::Win32::UI::WindowsAndMessaging::*;
// use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, LOGPIXELSX, LOGPIXELSY};
// use windows::Win32::Foundation::HWND;

use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use windows::Win32::Graphics::Gdi::{MONITOR_FROM_FLAGS, MonitorFromPoint};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE, DPI_AWARENESS_PER_MONITOR_AWARE, MONITOR_DPI_TYPE, GetDpiForMonitor, SetProcessDpiAwarenessContext, GetDpiAwarenessContextForProcess, GetAwarenessFromDpiAwarenessContext}; //DPI_AWARENESS, DPI_AWARENESS_CONTEXT, GetThreadDpiAwarenessContext
use windows::Win32::Foundation::{POINT, HANDLE};


mod rectangle;
mod winevent;

use ::uiexplore::signal_file;

mod uiexplore;
use uiexplore::{UITree, UIElementProps, UIElementInTree};

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

    let app_size_pos = AppContext::new_from_screen(0.4, 0.8);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
                    .with_inner_size([app_size_pos.app_width as f32, app_size_pos.app_height as f32])
                    .with_position(egui::Pos2::new(app_size_pos.app_left as f32, app_size_pos.app_top as f32))
                    .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "UI Explore",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(UIExplorer::new_with_state(app_size_pos, ui_tree)))
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
struct AppContext {
    screen_width: i32,
    screen_height: i32,
    screen_scale: f32,
    app_width: f32,
    app_height: f32,
    app_left: f32,
    app_top: f32,
}

impl AppContext {
    fn new(screen_width: i32, screen_height: i32, screen_scale: f32, app_width: f32, app_height: f32, app_left: f32, app_top: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            screen_scale,
            app_width,
            app_height,
            app_left,
            app_top,
        }
    }

    fn new_from_screen(horizontal_scaling: f32, vertical_scaling: f32) -> Self {
        
        let screen_size = get_system_metrics();
        let screen_width = screen_size.width;
        let screen_height = screen_size.height; 
        let screen_scale = get_screen_scale_factor();
        let app_width = screen_width as f32 * horizontal_scaling;
        let app_height = screen_height as f32 * vertical_scaling;
        let app_left = screen_width as f32 / 2.0 - app_width / 2.0;
        let app_top = screen_height as f32 / 2.0 - app_height / 2.0;
        Self::new(screen_width, screen_height, screen_scale, app_width, app_height, app_left, app_top)
    }
}

fn get_system_metrics() -> ScreenSize {
    unsafe {
        let x = GetSystemMetrics(SM_CXSCREEN);
        let y = GetSystemMetrics(SM_CYSCREEN);
        // println!("Screen size: {}x{}", x, y);
        ScreenSize { width: x, height: y }
    }
}

fn get_screen_scale_factor() -> f32 {

    unsafe {
        // First we need to set the DPI awareness context to per monitor aware
        // This is required to get the correct DPI for the monitor
        let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_FROM_FLAGS { 0: 2 });
        let _res_dpi_awareness_context = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE);
        let dpi_awareness_process = GetDpiAwarenessContextForProcess(HANDLE(std::ptr::null_mut()));
        let awareness_process = GetAwarenessFromDpiAwarenessContext(dpi_awareness_process);

        let awareness_fmt: String;
        let awareness = match awareness_process {
            DPI_AWARENESS_PER_MONITOR_AWARE => "Per Monitor Aware",
            _ => {
                awareness_fmt = format!("Unknown DPI Awareness: {:?}", awareness_process);
                awareness_fmt.as_str()
                },
        };

        let mut dpi_x = 0;
        let mut dpi_y = 0;
        let _res = GetDpiForMonitor(monitor, MONITOR_DPI_TYPE {0: 0}, &mut dpi_x, &mut dpi_y);


        // println!("DPI: ({}, {}), Awareness Process: {:?}", dpi_x, dpi_y, awareness);

        let x = GetSystemMetrics(SM_CXSCREEN);
        let y = GetSystemMetrics(SM_CYSCREEN);
        let scale_x = dpi_x as f32 / 96.0;
        let scale_y = dpi_y as f32 / 96.0;
        let scale = (scale_x + scale_y) / 2.0;
        println!("Screen size: {}x{}, DPI: {}x{}, Awareness Process: {}, Scale: {}", x, y, dpi_x, dpi_y, awareness, scale);

        scale
    }


}

fn launch_start_screen() {

    let msg: &str;

    let cmd = std::process::Command::new("start_screen.exe").spawn();

    match cmd {
        Ok(_) => { msg = "Start Screen successfully launched"; }
        Err(_) => { msg = "Failed to launch Start Screen"; }
    }

    printfmt!("{}", msg);
}

