// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(warnings)]

#[macro_use]
mod macros;

mod uiexplore;
use uiexplore::{UITree, UIElementProps};

use eframe::egui;
// use socarel::*;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

// #[allow(unused)]
pub type UIHashMap<K, V, S = std::hash::RandomState> = std::collections::HashMap<K, V, S>;
// #[allow(unused)]
type UIHashSet<T, S = std::hash::RandomState> = std::collections::HashSet<T, S>;
mod tree_map;
use tree_map::UITreeMap;


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]).with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "UI Explorer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(UIExplorer::new()))
        }),

    )
}

struct TreeState {
    active_element: Option<UIElementProps>,
}

impl TreeState {
    fn new() -> Self {
        Self {
            active_element: None,
        }
    }
}

#[allow(dead_code)]
struct UIExplorer {
    recording: bool,
    ui_tree: UITree,
    active_element: Option<UIElementProps>,
}

impl UIExplorer {
    fn new() -> Self {

        printfmt!("Getting UI Tree...");

        // get the ui tree in a separate thread
        let (tx, rx): (Sender<_>, Receiver<uiexplore::UITree>) = channel();
        thread::spawn(|| {
            uiexplore::get_all_elements(tx, None);
        });
        println!("Got ui tree - printing it now");        

        let ui_tree = rx.recv().unwrap();
        
        printfmt!("UI Tree retrieved...");

        Self {
            recording: false,
            ui_tree,
            active_element: None,
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
                //log::debug!("Rendering leaf node: {}", name);
                // Node has no children, so just show a label
                //ui.label(format! {"{}: {}", name, ui_element});
                if ui.label(format!("  {}", name)).clicked() {
                    state.active_element = Some(ui_element.clone());
                }
            }
            else {
                //log::debug!("Rendering new parent node: {}", name);
                // Render children under collapsing header
                if egui::CollapsingHeader::new(name)
                    .id_salt(format!("ch_node{}", child_index))
                    .show(ui, |ui| {
                        // Recursively render children
                        Self::render_ui_tree_recursive(ui, tree, child_index, state);
                    }).header_response.clicked() {
                        state.active_element = Some(ui_element.clone());
                    }
            }
        }
    }    

}

impl eframe::App for UIExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut state: TreeState;
        if let Some(tree_state) = &self.active_element {
            state = TreeState {active_element: Some(tree_state.clone()) };
        } else {
            state = TreeState::new();
        }

        egui::SidePanel::left("left_panel").show(ctx, |ui| { // .min_width(300.0).max_width(600.0)


            self.render_ui_tree(ui, &mut state);

        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("My egui Application");
            ui.horizontal(|ui| {
                // let name_label = ui.label("Your name: ");
                // ui.text_edit_singleline(&mut self.name)
                //     .labelled_by(name_label.id);

                if let Some(active_element) = &state.active_element {
                    egui::Grid::new("some_unique_id").min_col_width(100.0).show(ui, |ui| {
                        ui.label("Name:");
                        ui.label(active_element.name.clone());
                        ui.end_row();
                    
                        ui.label("Item Type:");
                        ui.label(active_element.item_type.clone());
                        ui.end_row();

                        ui.label("Localized Control Type:");
                        ui.label(active_element.localized_control_type.clone());
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

        self.active_element = state.active_element;
    }


}


