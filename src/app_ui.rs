use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;

use egui::{Color32, Response};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Foundation::POINT;

use crate::{rectangle, uiexplore, UIElementProps, UITree};
// use crate::rectangle::*;

#[derive(Clone)]
struct TreeState {
    active_element: Option<UIElementProps>,
    active_ui_element: Option<usize>,
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
pub struct UIExplorer {
    recording: bool,
    ui_tree: UITree,
    tree_state: Option<TreeState>,
    // active_element: Option<UIElementProps>,
    history: DeduplicatedHistory,
}

impl UIExplorer {
    #[allow(dead_code)]
    pub fn new() -> Self {

        // get the ui tree in a separate thread
        let (tx, rx): (Sender<_>, Receiver<uiexplore::UITree>) = channel();
        thread::spawn(|| {
            uiexplore::get_all_elements(tx, None);
        });

        let ui_tree = rx.recv().unwrap();

        Self {
            recording: false,
            ui_tree,
            tree_state: None,
            // active_element: None,
            history: DeduplicatedHistory::default(),
        }
    }

    pub fn new_with_state(ui_tree: UITree) -> Self {

        Self {
            recording: false,
            ui_tree,
            tree_state: None,
            // active_element: None,
            history: DeduplicatedHistory::default(),
        }
    }


    fn render_ui_tree(&mut self, ui: &mut egui::Ui, state: &mut TreeState, weak_bg_fill: Color32) {
        let tree = &self.ui_tree;
        // Display the file format as the root note, if there is one
        Self::render_ui_tree_recursive(ui, tree, 0, state, weak_bg_fill);
    }

    fn render_ui_tree_recursive(ui: &mut egui::Ui, tree: &UITree, idx: usize, state: &mut TreeState, weak_bg_fill: Color32) {
        
        for &child_index in tree.children(idx) {
            let (name, ui_element) = tree.node(child_index);

            // flag if this is the active element
            let mut is_active_element: bool = false;
            if let Some(active_id) = state.active_ui_element {
                if active_id == child_index {
                    is_active_element = true;
                }
            }

            if tree.children(child_index).is_empty() {
                // Node has no children, so just show a label
                let lbl = egui::Label::new(format!("  {}", name));
                let entry: Response;
                // let entry = ui.label(format!("  {}", name)).on_hover_cursor(egui::CursorIcon::Default);
                if is_active_element{
                    // show background to visually highlight the active element
                    let tmp_entry = egui::Frame::none()
                    .fill(weak_bg_fill)
                    .show(ui, |ui| {
                       ui.add(lbl).on_hover_cursor(egui::CursorIcon::Default);
                    });
                    entry = tmp_entry.response;
                } else {
                    // render standard label without any visual highlights
                    entry = ui.add(lbl).on_hover_cursor(egui::CursorIcon::Default);                    
                }
                
                if entry.clicked() {
                    state.active_element = Some(ui_element.clone());
                    state.active_ui_element = Some(child_index);
                }
                if entry.hovered() {
                    entry.highlight();                    
                }
            }
            else {
                // Render children under collapsing header
                let header: egui::CollapsingHeader;                
                // TODO: check if header is on path to active element, if yes open the header
                if "perform the check" != "perform the check 1" {
                    // header is not on path, render a standard CollapsingHeader
                    header = egui::CollapsingHeader::new(name)
                    .id_salt(format!("ch_node{}", child_index))
                } else {
                    if is_active_element {
                        // show background to visually highlight the active element
                        header = egui::CollapsingHeader::new(name)
                        .id_salt(format!("ch_node{}", child_index))
                        .default_open(true)
                        .show_background(true);
                    } else {
                        header = egui::CollapsingHeader::new(name)
                        .id_salt(format!("ch_node{}", child_index))
                        .default_open(true);
                        // TODO: or maybe better .open(Some(true)) ?? test it out...    
                    }
                }
                
                let header_resp = header
                    .show(ui, |ui| {
                        // Recursively render children
                        Self::render_ui_tree_recursive(ui, tree, child_index, state, weak_bg_fill);
                    });    
                    
                if header_resp.header_response.clicked() {
                    state.active_element = Some(ui_element.clone());
                    state.active_ui_element = Some(child_index);
                    
                }
            }
        }
    }    

}

impl eframe::App for UIExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut state: TreeState;
        if let Some(tree_state) = &self.tree_state { //.active_element 
            state = tree_state.clone();
        } else {
            state = TreeState::new();
        }        

        let weak_bg_fill = ctx.theme().default_visuals().widgets.inactive.weak_bg_fill;        

        egui::SidePanel::left("left_panel")
        .min_width(800.0)
        .max_width(1400.0)                
        .show(ctx, |ui| { // .min_width(300.0).max_width(600.0)

            egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                // printfmt!("running 'render_ui_tree' function on UIExplorer");
                self.render_ui_tree(ui, &mut state, weak_bg_fill);

            });

        });

        egui::TopBottomPanel::top("top_panel").resizable(true).show(ctx, |ui| {

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
                    
                    // for the visual event summary
                    let summary = event_summary(event, self.ui_tree.get_elements());
                    let full = format!("{event:#?}");
                    self.history.add(summary, full);

                    // update the actual active element
                    self.process_event(event, &mut state);
                }
            });
    
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.recording, "Recording");
                ui.label("Record events");
            });

            ui.add_space(8.0);

            self.history.ui(ui);

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
                        
                        ui.label("level:");
                        ui.label(active_element.level.to_string());
                        ui.end_row();
                        
                        ui.label("z-order:");
                        ui.label(active_element.z_order.to_string());
                        ui.end_row();

                    });    

                }
                else {
                    ui.label("No active element");
                }

            });

    
        });



        // self.active_element = state.active_element;
        self.tree_state = Some(state);
    }



}

impl UIExplorer {
    #[allow(dead_code)]
    fn process_event(&mut self, event: &egui::Event, state: &mut TreeState) {

        match event {
            egui::Event::MouseMoved { .. } => { 
                let cursor_position = unsafe {
                    let mut cursor_pos = POINT::default();
                    GetCursorPos(&mut cursor_pos).unwrap();
                    cursor_pos
                };
                                
                if let Some(ui_element_props) = rectangle::get_point_bounding_rect(&cursor_position, self.ui_tree.get_elements()) {
                    state.active_element = Some(ui_element_props.clone());
                } 
            }
            _ => (),
        }
    }
}


fn event_summary(event: &egui::Event, ui_elements: &Vec<UIElementProps>) -> String {
    match event {
        egui::Event::PointerMoved { .. }   => {        
            "PointerMoved { .. }".to_owned()
        }
        egui::Event::MouseMoved { .. } => { 
            let cursor_position = unsafe {
                let mut cursor_pos = POINT::default();
                GetCursorPos(&mut cursor_pos).unwrap();
                cursor_pos
            };

            if let Some(ui_element_props) = rectangle::get_point_bounding_rect(&cursor_position, ui_elements) {
                // format!("MouseMoved {{ x: {}, y: {} }} over {}", cursor_position.x, cursor_position.y, ui_element_props.name)
                format!("MouseMoved over {{ name: '{}', control_type: '{}' bounding_rect: {} }}", ui_element_props.name, ui_element_props.control_type, ui_element_props.bounding_rect)
            } else {
            // format!("MouseMoved {{ x: {}, y: {} }} ", cursor_position.x, cursor_position.y)
            "MouseMoved { .. }".to_owned()
            }
        }
        egui::Event::Zoom { .. } => "Zoom { .. }".to_owned(),
        egui::Event::Touch { phase, .. } => format!("Touch {{ phase: {phase:?}, .. }}"),
        egui::Event::MouseWheel { unit, .. } => format!("MouseWheel {{ unit: {unit:?}, .. }}"),

        _ => format!("{event:?}"),
    }
}
