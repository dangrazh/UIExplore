#![allow(dead_code)]


use crate::UITreeMap;


use std::sync::mpsc::Sender;

use uiautomation::core::UIAutomation;
use uiautomation::{UIElement, UITreeWalker};


#[derive(Debug, Clone)]
pub struct UITree {
    tree: UITreeMap<UIElementProps>,
    // ui_elements: Vec<UIElementProps>,
}

impl UITree {
    pub fn new(tree: UITreeMap<UIElementProps>) -> Self { // , ui_elements: Vec<UIElementProps>
        UITree {tree, } //ui_elements
    }

    pub fn get_tree(&self) -> &UITreeMap<UIElementProps> {
        &self.tree
    }

    // pub fn get_elements(&self) -> &Vec<UIElementProps> {
    //     &self.ui_elements
    // }

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
}

impl UIElementProps {
    pub fn new(from_element: UIElement) -> Self {
        UIElementProps::from(from_element)
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
            
        UIElementProps {
            name,
            classname,
            control_type,
            localized_control_type,
            framework_id,
            runtime_id,
            bounding_rect,        
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
    let mut tree = UITreeMap::new(item, UIElementProps::from(root.clone()));
    if let Ok(_first_child) = walker.get_first_child(&root) {     
        // itarate over all child ui elements
        get_element(&mut tree,  0, &walker, &root, 0, max_depth);
    }

    // pack the tree and ui_elements vector into a single struct
    let ui_tree = UITree::new(tree);

    // send the tree containing all UI elements back to the main thread
    tx.send(ui_tree).unwrap();

}


fn get_element(mut tree: &mut UITreeMap<UIElementProps>, parent: usize, walker: &UITreeWalker, element: &UIElement, level: usize, max_depth: Option<usize>)  {

    if let Some(limit) = max_depth {
        if level > limit {
            return;
        }    
    }
    
    let runtime_id = element.get_runtime_id().unwrap_or(vec![0, 0, 0, 0]).iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-");
    let item = format!("'{}' {} ({} | {} | {})", element.get_name().unwrap(), element.get_localized_control_type().unwrap(), element.get_classname().unwrap(), element.get_framework_id().unwrap(), runtime_id);
    
    let parent = tree.add_child(parent, item.as_str(), UIElementProps::from(element.clone()));

    // walking children now
    if let Ok(child) = walker.get_first_child(&element) {
        // getting child elements
        get_element(&mut tree, parent, walker, &child, level + 1, max_depth);

        let mut next = child;
        // walking siblings
        while let Ok(sibling) = walker.get_next_sibling(&next) {
            get_element(&mut tree, parent, walker, &sibling,  level + 1, max_depth);
            next = sibling;
        }
    }    
    
}
