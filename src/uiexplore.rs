#![allow(dead_code)]


use crate::macros;
use crate::UITreeMap;


use std::sync::mpsc::Sender;
// use std::sync::Arc;
// use std::sync::Mutex;

use socarel::*;

use uiautomation::core::UIAutomation;
use uiautomation::{Result, UIElement, UITreeWalker};


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
    pub item_type: String,
    pub localized_control_type: String,
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
        let item_type: String = item.get_item_type().unwrap_or("".to_string());
        let localized_control_type: String = item.get_localized_control_type().unwrap_or("".to_string());
        let runtime_id: Vec<i32> = item.get_runtime_id().unwrap_or(Vec::new());
        let bounding_rect: uiautomation::types::Rect = item.get_bounding_rectangle().unwrap_or(uiautomation::types::Rect::new(0, 0, 0, 0));
            
        UIElementProps {
            name,
            classname,
            item_type,
            localized_control_type,
            runtime_id,
            bounding_rect,        
        }
    }
}

pub struct UIElementNode {
    content: String,
    id: u32
}
 
impl UIElementNode {
    fn get_id(&self) -> u32 {
        self.id
    }
}
 
impl NodeContent for UIElementNode {
    // We parse the node content and return None if not a valid format
    fn new(content: &str) -> Option<Self> {
        let vec: Vec<&str> = content.split('§').collect();
        if vec.len() == 2 {
            match vec[0].trim().parse() {
                Ok(num) => Some(Self {
                    content: String::from(vec[1]),
                    id: num
                }),
                Err(_) => None
            }
        }
        else {
            None
        }
    }
 
    fn get_val(&self) -> &str {
        &self.content
    }
 
    fn gen_content(&self) -> String {
        format!("{}: {}", self.id, self.content)
    }
}


pub fn get_all_elements(tx: Sender<UITree>, max_depth: Option<usize>)  {   

    printfmt!("Getting all elements");
    
    let automation = UIAutomation::new().unwrap();
    let walker = automation.get_control_view_walker().unwrap();
    
    // get the desktop and all UI elements below the desktop
    let root = automation.get_root_element().unwrap();
    let runtime_id = root.get_runtime_id().unwrap_or(vec![0, 0, 0, 0]).iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-");
    let item = format!("'{}' {} ({} | {} | {})", root.get_name().unwrap(), root.get_localized_control_type().unwrap(), root.get_classname().unwrap(), root.get_framework_id().unwrap(), runtime_id);
    let mut tree = UITreeMap::new(item, UIElementProps::from(root.clone()));
    if let Ok(first_child) = walker.get_first_child(&root) {     
        // itarate over all child ui elements
        get_element(&mut tree,  0, &walker, &root, 0, max_depth);
    }

    printfmt!("done getting all elements, packing thins up in a UITree");
    // pack the tree and ui_elements vector into a single struct
    let ui_tree = UITree::new(tree);

    printfmt!("Sending UITree back to main thread");
    // send the tree containing all UI elements back to the main thread
    tx.send(ui_tree).unwrap();


}


fn get_element(mut tree: &mut UITreeMap<UIElementProps>, parent: usize, walker: &UITreeWalker, element: &UIElement, level: usize, max_depth: Option<usize>)  {

    if let Some(limit) = max_depth {
        if level > limit {
            // printfmt!("Level limit reached!");
            return;
        }    
    }
    
    // let element_name = element.get_name().unwrap();
    //printfmt!("{level} - getting element {element_name} name and converting to UIElementProps");



    // printfmt!("{level} - getting item label and adding it to the tree structure");

    let runtime_id = element.get_runtime_id().unwrap_or(vec![0, 0, 0, 0]).iter().map(|x| x.to_string()).collect::<Vec<String>>().join("-");
    let item = format!("'{}' {} ({} | {} | {})", element.get_name().unwrap(), element.get_localized_control_type().unwrap(), element.get_classname().unwrap(), element.get_framework_id().unwrap(), runtime_id);
    
    // printfmt!("{level} - got item {item} and adding it to the tree structure", );
    let parent = tree.add_child(parent, item.as_str(), UIElementProps::from(element.clone()));

    // printfmt!("{level} - added item to the tree structure");

    // printfmt!("{level} - walking children now");
    if let Ok(child) = walker.get_first_child(&element) {
        // printfmt!("{level} - getting child element");
        get_element(&mut tree, parent, walker, &child, level + 1, max_depth);

        let mut next = child;
        // printfmt!("{level} - walking siblings");
        while let Ok(sibling) = walker.get_next_sibling(&next) {
            // printfmt!("{level} - getting sibling element");
            get_element(&mut tree, parent, walker, &sibling,  level + 1, max_depth);

            next = sibling;
        }
    }    
    // printfmt!("{level} - done getting element");
    
}
