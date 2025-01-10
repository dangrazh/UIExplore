#![allow(dead_code)]


use crate::UITreeMap;


use std::sync::mpsc::Sender;

use uiautomation::core::UIAutomation;
use uiautomation::{UIElement, UITreeWalker};


#[derive(Debug, Clone)]
pub struct UITree {
    tree: UITreeMap<UIElementProps>,
    ui_elements: Vec<UIElementProps>,
}

impl UITree {
    pub fn new(tree: UITreeMap<UIElementProps>, ui_elements: Vec<UIElementProps>) -> Self {
        UITree {tree, ui_elements} 
    }

    pub fn get_tree(&self) -> &UITreeMap<UIElementProps> {
        &self.tree
    }

    pub fn get_elements(&self) -> &Vec<UIElementProps> {
        &self.ui_elements
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(usize, &UIElementProps),
    {
        self.tree.for_each(f);
    }

    pub fn root(&self) -> usize {
        self.tree.root()
    }

    pub fn children(&self, index: usize) -> &[usize] {
        self.tree.children(index)
    }

    pub fn node(&self, index: usize) -> (&str, &UIElementProps) {
        let node = &self.tree.node(index);
        (&node.name, &node.data)
    }

}


#[derive(Debug, Clone)]
pub struct UIElementProps {
    pub name: String,
    pub classname: String,
    pub control_type: String,
    pub localized_control_type: String,
    pub framework_id: String,
    pub runtime_id: Vec<i32>,
    pub bounding_rect: uiautomation::types::Rect,
    pub bounding_rect_size: i32,
    pub level: usize,
    pub z_order: usize,
}

impl UIElementProps {
    pub fn new(from_element: UIElement, level: usize, z_order: usize) -> Self {
        let mut elem = UIElementProps::from(from_element);
        elem.z_order = z_order;
        elem.level = level;
        elem
    }
}

impl From<UIElement> for UIElementProps {
    fn from(item: UIElement) -> Self {

        let name: String = item.get_name().unwrap_or("".to_string());
        let classname: String = item.get_classname().unwrap_or("".to_string());
        
        let mut control_type: String = "".to_string();
        if let Ok(ctrl_type) =  item.get_control_type() {
            control_type = ctrl_type.to_string();    
        }

        let localized_control_type: String = item.get_localized_control_type().unwrap_or("".to_string());
        let framework_id: String = item.get_framework_id().unwrap_or("".to_string());
        let runtime_id: Vec<i32> = item.get_runtime_id().unwrap_or(Vec::new());
        let bounding_rect: uiautomation::types::Rect = item.get_bounding_rectangle().unwrap_or(uiautomation::types::Rect::new(0, 0, 0, 0));
        let bounding_rect_size: i32 = (bounding_rect.get_right() - bounding_rect.get_left()) * (bounding_rect.get_bottom() - bounding_rect.get_top());            
        
        UIElementProps {
            name,
            classname,
            control_type,
            localized_control_type,
            framework_id,
            runtime_id,
            bounding_rect,
            bounding_rect_size,
            level: 0,
            z_order: 0,
        }
    }
}

pub fn get_all_elements(tx: Sender<UITree>, max_depth: Option<usize>)  {   
    
    let automation = UIAutomation::new().unwrap();
    let walker = automation.get_control_view_walker().unwrap();
    
    // get the desktop and all UI elements below the desktop
    let root = automation.get_root_element().unwrap();
    let runtime_id = root.get_runtime_id().unwrap_or(vec![0, 0, 0, 0]).iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-");
    let item = format!("'{}' {} ({} | {} | {})", root.get_name().unwrap(), root.get_localized_control_type().unwrap(), root.get_classname().unwrap(), root.get_framework_id().unwrap(), runtime_id);
    let ui_elem_props = UIElementProps::new(root.clone(), 0, 999);
    let mut tree = UITreeMap::new(item, ui_elem_props.clone());
    let mut ui_elements: Vec<UIElementProps> = vec![ui_elem_props];
    
    // printfmt!("Root element: {}", debug_clone.name);
    if let Ok(_first_child) = walker.get_first_child(&root) {     
        // itarate over all child ui elements
        get_element(&mut tree, &mut ui_elements,  0, &walker, &root, 0, 0, max_depth);
    }

    // sorting the elements by z_order and then by ascending size of the bounding rectangle
    ui_elements.sort_by(|a, b| a.bounding_rect_size.cmp(&b.bounding_rect_size));
    ui_elements.sort_by(|a, b| a.z_order.cmp(&b.z_order));

    // pack the tree and ui_elements vector into a single struct
    let ui_tree = UITree::new(tree, ui_elements);

    // send the tree containing all UI elements back to the main thread
    tx.send(ui_tree).unwrap();

}


fn get_element(mut tree: &mut UITreeMap<UIElementProps>, mut ui_elements: &mut Vec<UIElementProps>, parent: usize, walker: &UITreeWalker, element: &UIElement, level: usize, mut z_order: usize, max_depth: Option<usize>)  {

    if let Some(limit) = max_depth {
        if level > limit {
            return;
        }    
    }

    let runtime_id = element.get_runtime_id().unwrap_or(vec![0, 0, 0, 0]).iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-");
    let item = format!("'{}' {} ({} | {} | {})", element.get_name().unwrap(), element.get_localized_control_type().unwrap(), element.get_classname().unwrap(), element.get_framework_id().unwrap(), runtime_id);
    let ui_elem_props: UIElementProps;

    if level == 0 {
        // manually setting the z_order for the root element
        ui_elem_props = UIElementProps::new(element.clone(), level, 999);
    } else {
        ui_elem_props = UIElementProps::new(element.clone(), level, z_order);
    }
    
    let parent = tree.add_child(parent, item.as_str(), ui_elem_props.clone());
    ui_elements.push(ui_elem_props);

    // walking children now
    if let Ok(child) = walker.get_first_child(&element) {
        // getting child elements
        get_element(&mut tree, &mut ui_elements, parent, walker, &child, level + 1, z_order, max_depth);
        let mut next = child;
        // walking siblings
        while let Ok(sibling) = walker.get_next_sibling(&next) {
            // incrementing z_order for each sibling
            if level + 1 == 1 {
                z_order += 1;
            }
            get_element(&mut tree, &mut ui_elements, parent, walker, &sibling,  level + 1, z_order, max_depth);
            next = sibling;
        }
    }    
    
}
