#[macro_use]
extern crate lazy_static;

use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use str_helper::StrHelper;
pub use tscn_helper::{Node, NodeType, TscnHelper, VarType};

// pub mod nodes;
pub mod str_helper;
pub mod tscn_helper;

type PropertyMap = HashMap<String, VarType>;

#[derive(Debug, Clone)]
pub struct NodeEntry {
    pub uuid: u16,
    pub level: usize,
    pub name: String,
    pub rtype: String,
    pub parent_id: usize,
    pub node_type: NodeType,
    pub childrens: Vec<usize>,
    pub properties: PropertyMap,
}

#[derive(Debug, Clone)]
pub struct Tscn {
    pub rtype: String,
    pub nodes: HashMap<usize, NodeEntry>,
    pub resource: PropertyMap,
    pub sub_resources: HashMap<usize, NodeEntry>,
    pub ext_resources: HashMap<usize, Tscn>,
}

impl Default for NodeEntry {
    fn default() -> Self {
        NodeEntry {
            uuid: 0,
            name: "".to_string(),
            level: 0,
            rtype: "".to_string(),
            parent_id: 0,
            childrens: Vec::new(),
            node_type: NodeType::Node,
            properties: HashMap::new(),
        }
    }
}

impl NodeEntry {
    fn new_type(rtype: &str) -> Self {
        NodeEntry {
            rtype: rtype.to_string(),
            ..NodeEntry::default()
        }
    }
}

pub trait PropertyTrait {
    /// Inserts value into path (i.e. `into/path`)
    fn insert_to(&mut self, path: String, value: VarType);
    /// Gets value from path (i.e. `from/path`)
    fn get_from(&self, path: &String) -> Option<&VarType>;
    /// Get value mutably from path (i.e. `from/path`)
    fn get_from_mut(&mut self, path: &String) -> Option<&mut VarType>;
}

impl PropertyTrait for PropertyMap {
    fn insert_to(&mut self, path: String, value: VarType) {
        let split: Vec<&str> = path.split('/').collect();
        if split.len() == 1 {
            self.insert(path, value);
        } else if split.len() >= 2 {
            let hmap: Option<&mut PropertyMap> =
                if let Some(VarType::Map(map)) = self.get_mut(split[0]) {
                    Some(map)
                } else {
                    self.insert(split[0].to_string(), VarType::Map(HashMap::new()));
                    if let Some(VarType::Map(map)) = self.get_mut(&split[0].to_string()) {
                        Some(map)
                    } else {
                        None
                    }
                };

            if let Some(map) = hmap {
                map.insert_to(split[1..].join("/"), value);
            }
        }
    }

    fn get_from(&self, path: &String) -> Option<&VarType> {
        let split: Vec<&str> = path.split('/').collect();

        if split.len() == 1 {
            self.get(path)
        } else {
            if let VarType::Map(map) = self.get(path)? {
                map.get_from(&split[1..].join("/"))
            } else {
                None
            }
        }
    }

    fn get_from_mut(&mut self, path: &String) -> Option<&mut VarType> {
        let split: Vec<&str> = path.split('/').collect();

        if split.len() == 1 {
            self.get_mut(path)
        } else if split.len() >= 2 {
            if let VarType::Map(map) = self.get_mut(split[0])? {
                map.get_from_mut(&split[1..].join("/"))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct Loader<'a> {
    map_path: HashMap<String, &'a Path>,
    ctx: IndexMap<String, usize>,
    context: Option<Node>,
    node_id: usize,
    rtype: String,
    resource: HashMap<String, VarType>,
    sub_resources: HashMap<usize, NodeEntry>,
    ext_resources: HashMap<usize, Tscn>,
    nodes: HashMap<usize, NodeEntry>,
    last_prop: Option<String>,
}

impl<'a> Loader<'a> {
    pub fn new() -> Self {
        let map_path = HashMap::new();

        let context: Option<Node> = None;
        let ctx: IndexMap<String, usize> = IndexMap::new();

        let rtype = "Scene".to_string();
        let resource: HashMap<String, VarType> = HashMap::new();
        let nodes: HashMap<usize, NodeEntry> = HashMap::new();
        let sub_resources = HashMap::new();
        let ext_resources: HashMap<usize, Tscn> = HashMap::new();

        let node_id: usize = 0;
        let last_prop: Option<String> = None;

        Loader {
            map_path,
            ctx,
            context,
            node_id,
            rtype,
            resource,
            sub_resources,
            ext_resources,
            nodes,
            last_prop,
        }
    }

    pub fn register_path(&mut self, gdpath: String, syspath: &'a Path) {
        self.map_path.insert(gdpath, syspath);
    }

    pub fn load(&mut self, gdpath: String) -> Option<Tscn> {
        if let Some(path) = self.get_path(gdpath) {
            let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
            return Some(self.parse_tscn(&contents));
        }

        None
    }

    fn clone_loader(&self) -> Self {
        let mut loader = Loader::new();
        loader.map_path = self.map_path.clone();

        loader
    }

    fn get_path(&self, gdpath: String) -> Option<PathBuf> {
        for (spath, syspath) in &self.map_path {
            if gdpath.len() >= spath.len() + 1 && spath.as_str() == &gdpath[0..spath.len()] {
                let jpath = &gdpath[spath.len() + 1..];
                let joined_path = syspath.join(jpath);

                if joined_path.exists() {
                    return Some(joined_path);
                }
            }
        }

        None
    }

    fn parse_node(&mut self, line: &str) {
        let (node_type, attributes) = TscnHelper::parse_node(line);
        self.context = Some(TscnHelper::get_node(node_type, attributes));

        if self.context.is_none() {
            return;
        }

        let node = self.context.clone().unwrap();
        match node.node_type {
            NodeType::SubResource => {
                self.sub_resources
                    .insert(node.id, NodeEntry::new_type(&node.rtype));
            }

            NodeType::Node => {
                let entry: NodeEntry = if node.parent == String::from("") {
                    self.ctx.insert(".".to_string(), self.node_id);

                    NodeEntry {
                        name: node.name,
                        rtype: node.rtype,
                        ..NodeEntry::default()
                    }
                } else {
                    let clonned_ctx = self.ctx.clone();
                    let (level, _, parent_id) = clonned_ctx
                        .get_full(node.parent.split("/").last().unwrap())
                        .unwrap();

                    let last_id = self.ctx.len() - 1;
                    if level != last_id {
                        for _ in level..last_id {
                            self.ctx.pop();
                        }
                    }

                    self.ctx.insert(node.name.clone(), self.node_id);
                    self.nodes
                        .get_mut(parent_id)
                        .expect("Missing parent node")
                        .childrens
                        .push(self.node_id);

                    NodeEntry {
                        uuid: TscnHelper::get_path_hash(&self.ctx, &self.nodes),
                        level: level + 1,
                        name: node.name,
                        rtype: node.rtype,
                        parent_id: *parent_id,
                        ..NodeEntry::default()
                    }
                };

                self.nodes.insert(self.node_id, entry);
                self.node_id += 1;
            }

            NodeType::GdResource => {
                self.rtype = node.rtype;
            }

            NodeType::Resource => (),
            NodeType::GdScene => (),
            NodeType::ExtResource => {
                let mut loader = self.clone_loader();
                let load = loader.load(node.path);

                if let Some(tscn) = load {
                    self.ext_resources.insert(node.id, tscn);
                }
            }
        }
    }

    fn get_ctxnode_props(&mut self) -> Option<&mut PropertyMap> {
        if let Some(node) = self.context.clone() {
            match node.node_type {
                NodeType::Node => {
                    let (_, id) = self.ctx.get_index(self.ctx.len() - 1).unwrap();
                    Some(&mut self.nodes.get_mut(id).unwrap().properties)
                }
                NodeType::SubResource => {
                    Some(&mut self.sub_resources.get_mut(&node.id).unwrap().properties)
                }
                NodeType::Resource => Some(&mut self.resource),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn parse_line(&mut self, line: &str) {
        if line.is_empty() || line.trim() == "}" || line.trim_matches(' ') == "}]" {
            return;
        }

        let lprop_clone = self.last_prop.clone();

        // If it is node block definition
        if line.check_borders('[', ']') {
            self.parse_node(&line);
        } else if let Some(ctxprops) = self.get_ctxnode_props() {
            if let Some(command) = TscnHelper::parse_command(line) {
                ctxprops.insert_to(command.lhs.clone(), command.rhs);
                self.last_prop = Some(command.lhs);
            } else if let Some(command) = TscnHelper::parse_obj(line) {
                if let Some(prop) = lprop_clone {
                    if let Some(VarType::Map(obj)) = ctxprops.get_from_mut(&prop) {
                        obj.insert(command.lhs, command.rhs);
                    }
                }
            }
        }
    }

    pub fn parse_tscn(&mut self, tscn: &str) -> Tscn {
        for line in tscn.lines() {
            self.parse_line(line);
        }

        Tscn {
            nodes: self.nodes.clone(),
            rtype: self.rtype.clone(),
            resource: self.resource.clone(),
            sub_resources: self.sub_resources.clone(),
            ext_resources: self.ext_resources.clone(),
        }
    }
}
