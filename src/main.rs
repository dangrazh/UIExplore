#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// #![allow(warnings)]

#[macro_use]
mod macros;

// mod geometry;
// use geometry::is_inside_rectancle;

mod uiexplore;
use uiexplore::{UITree, UIElementProps};

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

    let ui_tree = rx.recv().unwrap();
    
    printfmt!("UI Tree retrieved, setting up UIExplorer app...");

    
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 800.0]).with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "UI Explorer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(UIExplorer::new_with_state(ui_tree)))
        }),

    )
}

struct TreeState {
    active_element: Option<UIElementProps>,
    active_ui_element: Option<egui::Id>,
}

impl TreeState {
    fn new() -> Self {
        Self {
            active_element: None,
            active_ui_element: None,
        }
    }
}

struct HistoryEntry {
    summary: String,
    entries: Vec<String>,
}

#[derive(Default)]
struct DeduplicatedHistory {
    history: std::collections::VecDeque<HistoryEntry>,
}

impl DeduplicatedHistory {
    fn add(&mut self, summary: String, full: String) {
        if let Some(entry) = self.history.back_mut() {
            if entry.summary == summary {
                entry.entries.push(full);
                return;
            }
        }
        self.history.push_back(HistoryEntry {
            summary,
            entries: vec![full],
        });
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }

    fn ui(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                for HistoryEntry { summary, entries } in self.history.iter().rev() {
                    ui.horizontal(|ui| {
                        let response = ui.code(summary);
                        if entries.len() < 2 {
                            response
                        } else {
                            response | ui.weak(format!(" x{}", entries.len()))
                        }
                    })
                    .inner
                    .on_hover_ui(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        for entry in entries.iter().rev() {
                            ui.code(entry);
                        }
                    });
                }
            });
    }

}


// #[allow(dead_code)]
struct UIExplorer {
    recording: bool,
    ui_tree: UITree,
    active_element: Option<UIElementProps>,
    history: DeduplicatedHistory,
}

impl UIExplorer {
    #[allow(dead_code)]
    fn new() -> Self {

        // get the ui tree in a separate thread
        let (tx, rx): (Sender<_>, Receiver<uiexplore::UITree>) = channel();
        thread::spawn(|| {
            uiexplore::get_all_elements(tx, None);
        });

        let ui_tree = rx.recv().unwrap();

        Self {
            recording: false,
            ui_tree,
            active_element: None,
            history: DeduplicatedHistory::default(),
        }
    }

    fn new_with_state(ui_tree: UITree) -> Self {

        Self {
            recording: false,
            ui_tree,
            active_element: None,
            history: DeduplicatedHistory::default(),
        }
    }


    fn render_ui_tree(&mut self, ui: &mut egui::Ui, state: &mut TreeState) {
        let tree = &self.ui_tree;
        // Display the file format as the root note, if there is one
        Self::render_ui_tree_recursive(ui, tree, 0, state);
    }

    pub fn render_ui_tree_recursive(ui: &mut egui::Ui, tree: &UITree, idx: usize, state: &mut TreeState) {
        for &child_index in tree.children(idx) {
            let (name, ui_element) = tree.node(child_index);

            if tree.children(child_index).is_empty() {
                // Node has no children, so just show a label
                let entry = ui.label(format!("  {}", name)).on_hover_cursor(egui::CursorIcon::Default);
                if entry.clicked() {
                    state.active_element = Some(ui_element.clone());
                }
                if entry.hovered() {
                    entry.highlight();                    
                }
            }
            else {
                // Render children under collapsing header
                let header = egui::CollapsingHeader::new(name)
                    .id_salt(format!("ch_node{}", child_index));
                let header_resp = header
                    .show(ui, |ui| {
                        // Recursively render children
                        Self::render_ui_tree_recursive(ui, tree, child_index, state);
                    });
                if header_resp.header_response.clicked() {
                    state.active_element = Some(ui_element.clone());
                    state.active_ui_element = Some(header_resp.header_response.id);
                    
                }
            }
        }
    }    

}

impl eframe::App for UIExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut state: TreeState;
        if let Some(tree_state) = &self.active_element {
            state = TreeState {active_element: Some(tree_state.clone()), active_ui_element: None };
        } else {
            state = TreeState::new();
        }

        egui::SidePanel::left("left_panel").min_width(800.0).show(ctx, |ui| { // .min_width(300.0).max_width(600.0)

            egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {

                self.render_ui_tree(ui, &mut state);

            });

        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
                
            ui.horizontal(|ui| {

                if let Some(active_element) = &state.active_element {
                    egui::Grid::new("some_unique_id").min_col_width(100.0).show(ui, |ui| {
                        ui.label("Name:");
                        ui.label(active_element.name.clone());
                        ui.end_row();
                    
                        ui.label("Control Type:");
                        ui.label(active_element.control_type.clone());
                        ui.end_row();

                        ui.label("Localized Control Type:");
                        ui.label(active_element.localized_control_type.clone());
                        ui.end_row();

                        ui.label("Framework ID:");
                        ui.label(active_element.framework_id.clone());
                        ui.end_row();

                        ui.label("Class Name:");
                        ui.label(active_element.classname.clone());
                        ui.end_row();

                        ui.label("Runtime ID:");
                        ui.label(active_element.runtime_id.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-"));
                        ui.end_row();

                        ui.label("Surrounding Rectangle:");
                        ui.label(format!("{:?}", active_element.bounding_rect));
                        ui.end_row();
                        
                    });    

                }
                else {
                    ui.label("No active element");
                }

            });

    
        });

        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).show(ctx, |ui| {

            ui.input(|i| {
                for event in &i.raw.events {
    
                    if !self.recording && matches!(
                        event,
                        egui::Event::PointerMoved { .. }
                            | egui::Event::MouseMoved { .. }
                            | egui::Event::Touch { .. }
                    )
                {
                    continue;
                }
                    
                    let summary = event_summary(event);
                    let full = format!("{event:#?}");
                    self.history.add(summary, full);
    
                }
            });
    
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.recording, "Recording");
                ui.label("Record events");
            });

            ui.add_space(8.0);

            self.history.ui(ui);

        });


        self.active_element = state.active_element;
    }


}


fn event_summary(event: &egui::Event) -> String {
    match event {
        egui::Event::PointerMoved { .. } => "PointerMoved { .. }".to_owned(),
        egui::Event::MouseMoved { .. } => "MouseMoved { .. }".to_owned(),
        egui::Event::Zoom { .. } => "Zoom { .. }".to_owned(),
        egui::Event::Touch { phase, .. } => format!("Touch {{ phase: {phase:?}, .. }}"),
        egui::Event::MouseWheel { unit, .. } => format!("MouseWheel {{ unit: {unit:?}, .. }}"),

        _ => format!("{event:?}"),
    }
}