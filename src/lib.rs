#[macro_use]
extern crate lazy_static;

use crate::tscn_helper::Node;
use indexmap::IndexMap;
use std::collections::HashMap;

use str_helper::StrHelper;
use tscn_helper::{NodeType, TscnHelper, VarType};

pub mod str_helper;
pub mod tscn_helper;

#[derive(Debug, Clone)]
pub struct NodeEntry {
    pub uuid: u16,
    pub level: usize,
    pub name: String,
    pub rtype: String,
    pub parent_id: usize,
    pub node_type: NodeType,
    pub childrens: Vec<usize>,
    pub properties: HashMap<String, VarType>,
}

#[derive(Debug, Clone)]
pub struct Tscn {
    pub nodes: HashMap<usize, NodeEntry>,
    pub sub_resources: HashMap<usize, NodeEntry>,
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

fn parse_node(
    line: &str,
    ctx: &mut IndexMap<String, usize>,
    context: &mut Option<Node>,
    sub_resources: &mut HashMap<usize, NodeEntry>,
    nodes: &mut HashMap<usize, NodeEntry>,
    node_id: &mut usize,
) {
    let (node_type, attributes) = TscnHelper::parse_node(line);
    *context = Some(TscnHelper::get_node(node_type, attributes));

    if context.is_none() {
        return;
    }

    let node = context.clone().unwrap();
    match node.node_type {
        NodeType::SubResource => {
            sub_resources.insert(node.id, NodeEntry::new_type(&node.rtype));
        }
        NodeType::Node => {
            let entry: NodeEntry = if node.parent == String::from("") {
                ctx.insert(".".to_string(), *node_id);

                NodeEntry {
                    name: node.name,
                    rtype: node.rtype,
                    ..NodeEntry::default()
                }
            } else {
                let clonned_ctx = ctx.clone();
                let (level, _, parent_id) = clonned_ctx
                    .get_full(node.parent.split("/").last().unwrap())
                    .unwrap();

                let last_id = ctx.len() - 1;
                if level != last_id {
                    for _ in level..last_id {
                        ctx.pop();
                    }
                }

                ctx.insert(node.name.clone(), *node_id);
                nodes
                    .get_mut(parent_id)
                    .expect("Missing parent node")
                    .childrens
                    .push(*node_id);

                NodeEntry {
                    uuid: TscnHelper::get_path_hash(&ctx),
                    level: level + 1,
                    name: node.name,
                    rtype: node.rtype,
                    parent_id: *parent_id,
                    ..NodeEntry::default()
                }
            };

            nodes.insert(*node_id, entry);
            *node_id += 1;
        }
        NodeType::GdScene => (),
        NodeType::ExtResource => (),
    }
}

pub fn parse_tscn(tscn: &str) -> Tscn {
    let mut context: Option<Node> = None;
    let mut ctx: IndexMap<String, usize> = IndexMap::new();

    let mut nodes: HashMap<usize, NodeEntry> = HashMap::new();
    let mut sub_resources = HashMap::new();

    let mut node_id: usize = 0;
    let mut last_prop: Option<String> = None;

    for line in tscn.lines() {
        if line.is_empty() {
            continue;
        }

        // If it is node block definition
        if line.check_borders('[', ']') {
            parse_node(
                &line,
                &mut ctx,
                &mut context,
                &mut sub_resources,
                &mut nodes,
                &mut node_id,
            );
        } else if context.is_none() || line.trim() == "}" || line.trim_matches(' ') == "}]" {
            continue;
        } else if let Some(command) = TscnHelper::parse_command(line) {
            let node = context.clone().unwrap();
            let lhs = command.lhs.clone();

            match node.node_type {
                NodeType::Node => {
                    let (_, id) = ctx.get_index(ctx.len() - 1).unwrap();
                    nodes
                        .get_mut(id)
                        .unwrap()
                        .properties
                        .insert(lhs, command.rhs);
                    last_prop = Some(command.lhs.clone());
                }
                NodeType::SubResource => {
                    sub_resources
                        .get_mut(&node.id)
                        .unwrap()
                        .properties
                        .insert(lhs, command.rhs);
                    last_prop = Some(command.lhs.clone());
                }
                NodeType::ExtResource => (),
                NodeType::GdScene => (),
            };
        } else if let Some(command) = TscnHelper::parse_obj(line) {
            if let Some(prop) = last_prop.clone() {
                let node = context.clone().unwrap();

                match node.node_type {
                    NodeType::Node => {
                        let (_, id) = ctx.get_index(ctx.len() - 1).unwrap();

                        if let Some(VarType::Map(obj)) =
                            nodes.get_mut(id).unwrap().properties.get_mut(&prop)
                        {
                            obj.insert(command.lhs, command.rhs);
                        }
                    }
                    NodeType::SubResource => {
                        if let Some(VarType::Map(obj)) = sub_resources
                            .get_mut(&node.id)
                            .unwrap()
                            .properties
                            .get_mut(&prop)
                        {
                            obj.insert(command.lhs, command.rhs);
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    Tscn {
        nodes,
        sub_resources,
    }
}
