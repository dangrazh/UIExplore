//! A generic tree structure with fast key-value lookup (not collision safe!)
#![allow(dead_code)]
use crate::{UIHashMap, UIHashSet};

// A generic node in a UITreeMap
#[derive(Debug, Clone)]
pub struct UITreeNode<T> {
    pub name: String,
    pub index: usize,
    pub parent: usize,
    pub children: Vec<usize>,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct UITreeMap<T> {
    nodes: Vec<UITreeNode<T>>,
    name_to_index: UIHashMap<String, usize>, // Name-to-index map for optional lookups
}

impl<T> UITreeMap<T> {
    pub fn new(root_name: String, root_data: T) -> Self {
        let root = UITreeNode {
            name: root_name.clone(),
            index: 0,
            parent: 0,
            children: Vec::new(),
            data: root_data,
        };

        let mut name_to_index = UIHashMap::default();
        name_to_index.insert(root_name, 0);

        Self {
            nodes: vec![root],
            name_to_index,
        }
    }

    pub fn root(&self) -> usize {
        0 // Root is always index 0
    }

    pub fn children(&self, index: usize) -> &[usize] {
        &self.nodes[index].children
    }

    pub fn node(&self, index: usize) -> &UITreeNode<T> {
        &self.nodes[index]
    }

    pub fn add_child(&mut self, parent: usize, name: &str, data: T) -> usize {
        let index = self.nodes.len();
        let node = UITreeNode {
            name: name.to_string(),
            index,
            parent,
            children: Vec::new(),
            data,
        };

        self.name_to_index.insert(name.to_string(), index);
        self.nodes[parent].children.push(index);
        self.nodes.push(node);
        index
    }

    pub fn get_path_to_element(&self, index: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current_index = index;
        while current_index != 0 {
            path.push(current_index);
            current_index = self.nodes[current_index].parent;
        }
        path.reverse(); // Reverse to get the path from root to the node
        path
    }

    /// Walks the tree and calls the callback on each node's data, immutably
    pub fn for_each<F>(&self, mut callback: F)
    where
        F: FnMut(usize, &T),
    {
        let mut visited = UIHashSet::new();
        self.for_each_recursive(self.root(), &mut callback, &mut visited);
    }

    /// Internal helper for recursive traversal.
    fn for_each_recursive<F>(&self, index: usize, callback: &mut F, visited: &mut UIHashSet<usize>)
    where
        F: FnMut(usize, &T),
    {
        if visited.contains(&index) {
            return; // Prevent cycles
        }
        visited.insert(index);

        let node = &self.nodes[index];
        callback(index, &node.data);

        for &child in &node.children {
            self.for_each_recursive(child, callback, visited);
        }
    }

    pub fn debug_tree<F>(&self, index: usize, indent: usize, display: &F, visited: &mut UIHashSet<usize>)
    where
        F: Fn(&T) -> String,
    {
        if visited.contains(&index) {
            println!("{}(Cycle detected at node {})", " ".repeat(indent), index);
            return;
        }
        visited.insert(index);

        let node = &self.nodes[index];
        let prefix = " ".repeat(indent);
        println!("{}{}: {}", prefix, &node.name, display(&node.data));

        for &child in &node.children {
            self.debug_tree(child, indent + 2, display, visited);
        }
    }

    pub fn debug_with<F>(&self, f: &mut std::fmt::Formatter<'_>, display: &F) -> std::fmt::Result
    where
        F: Fn(&T) -> String,
    {
        let mut visited = UIHashSet::default();
        self.debug_fmt_node_with(f, self.root(), 0, display, &mut visited)
    }

    fn debug_fmt_node_with<F>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        index: usize,
        indent: usize,
        display: &F,
        visited: &mut UIHashSet<usize>,
    ) -> std::fmt::Result
    where
        F: Fn(&T) -> String,
    {
        if visited.contains(&index) {
            writeln!(f, "{}(Cycle detected at node {})", " ".repeat(indent), index)?;
            return Ok(());
        }
        visited.insert(index);

        let node = &self.nodes[index];
        let prefix = " ".repeat(indent);
        writeln!(f, "{}{}: {}", prefix, node.name, display(&node.data))?;

        for &child in &node.children {
            self.debug_fmt_node_with(f, child, indent + 2, display, visited)?;
        }

        Ok(())
    }
}

pub trait UITree {
    type Data;

    fn tree_mut(&mut self) -> &mut UITreeMap<Self::Data>;
    fn tree(&self) -> &UITreeMap<Self::Data>;

    fn root(&self) -> usize {
        0
    }

    fn add_child<'a>(&'a mut self, parent: usize, name: &str, data: Self::Data) -> UITreeCursor<'a, Self::Data> {
        let child_index = self.tree_mut().add_child(parent, name, data);
        UITreeCursor {
            tree: self.tree_mut(),
            parent_index: parent,
            current_index: child_index,
        }
    }

    fn debug_tree(&self, display: impl Fn(&Self::Data) -> String) {
        let mut visited = UIHashSet::new();
        self.tree().debug_tree(self.root(), 0, &display, &mut visited);
    }
}

// Cursor for chaining child and sibling additions
pub struct UITreeCursor<'a, T> {
    tree: &'a mut UITreeMap<T>,
    parent_index: usize,
    current_index: usize,
}

impl<'a, T: Default> UITreeCursor<'a, T> {
    pub fn new(tree: &'a mut UITreeMap<T>, parent_index: usize, current_index: usize) -> Self {
        Self {
            tree,
            parent_index,
            current_index,
        }
    }

    /// Add a child to the current node.
    pub fn add_child(mut self, name: &str, data: T) -> Self {
        let child_index = self.tree.add_child(self.current_index, name, data);
        self.parent_index = self.current_index;
        self.current_index = child_index;
        self
    }

    /// Add a sibling to the current node.
    pub fn add_sibling(mut self, name: &str, data: T) -> Self {
        let sibling_index = self.tree.add_child(self.parent_index, name, data);
        self.current_index = sibling_index;
        self
    }

    /// Return the parent node of this node.
    pub fn up(mut self) -> Self {
        let parent_node = self.tree.node(self.parent_index);
        self.parent_index = parent_node.parent;
        self.current_index = parent_node.index;
        self
    }

    /// Return the index of the current node.
    pub fn index(&self) -> usize {
        self.current_index
    }
}