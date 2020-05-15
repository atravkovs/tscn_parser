#[macro_use]
extern crate lazy_static;

use crate::tscn_helper::Node;
use indexmap::IndexMap;
use std::collections::HashMap;

use str_helper::StrHelper;
use tscn_helper::{Command, NodeType, TscnHelper, VarType};

pub mod str_helper;
pub mod tscn_helper;

#[derive(Debug, Clone)]
pub struct SubResourceEntry {
    pub rtype: String,
    pub properties: HashMap<String, VarType>,
}

#[derive(Debug, Clone)]
pub struct NodeEntry {
    pub uuid: u16,
    pub level: usize,
    pub name: String,
    pub rtype: String,
    pub parent_id: usize,
    pub properties: HashMap<String, VarType>,
    pub childrens: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Tscn {
    pub nodes: HashMap<usize, NodeEntry>,
    pub sub_resources: HashMap<usize, SubResourceEntry>,
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
            properties: HashMap::new(),
        }
    }
}

impl SubResourceEntry {
    fn new(rtype: &str) -> Self {
        SubResourceEntry {
            rtype: rtype.to_string(),
            properties: HashMap::new(),
        }
    }
}

pub fn parse_tscn(tscn: &str) -> Tscn {
    let mut context: Option<Node> = None;
    let mut ctx: IndexMap<String, usize> = IndexMap::new();

    let mut nodes: HashMap<usize, NodeEntry> = HashMap::new();
    let mut sub_resources = HashMap::new();

    let mut node_id: usize = 0;

    for line in tscn.lines() {
        if line.is_empty() {
            continue;
        }

        // If it is node block definition
        if line.check_borders('[', ']') {
            let (node_type, attributes) = TscnHelper::parse_node(line);
            println!("{:?}", line);
            context = Some(TscnHelper::get_node(node_type, attributes));

            if context.is_none() {
                continue;
            }

            let node = context.clone().unwrap();
            match node.node_type {
                NodeType::SubResource => {
                    sub_resources.insert(node.id, SubResourceEntry::new(&node.rtype));
                }
                NodeType::Node => {
                    println!("HOPA");
                    let entry: NodeEntry = if node.parent == String::from("") {
                        ctx.insert(".".to_string(), node_id);

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

                        println!("Add name {} id {}", node.name, node_id);
                        ctx.insert(node.name.clone(), node_id);
                        nodes
                            .get_mut(parent_id)
                            .expect("Missing parent node")
                            .childrens
                            .push(node_id);

                        NodeEntry {
                            uuid: TscnHelper::get_path_hash(&ctx),
                            level: level + 1,
                            name: node.name,
                            rtype: node.rtype,
                            parent_id: *parent_id,
                            ..NodeEntry::default()
                        }
                    };

                    nodes.insert(node_id, entry);
                    node_id += 1;
                }
                NodeType::GdScene => (),
                NodeType::ExtResource => (),
            }

            continue;
        }

        if context.is_none() {
            continue;
        }

        let command: Command = TscnHelper::parse_command(line);

        let node = context.clone().unwrap();
        match node.node_type {
            NodeType::Node => {
                println!("{:?}", node.rtype);
                let (_, id) = ctx.get_index(ctx.len() - 1).unwrap();
                nodes
                    .get_mut(id)
                    .unwrap()
                    .properties
                    .insert(command.lhs, command.rhs);
            }
            NodeType::SubResource => {
                sub_resources
                    .get_mut(&node.id)
                    .unwrap()
                    .properties
                    .insert(command.lhs, command.rhs);
            }
            NodeType::ExtResource => (),
            NodeType::GdScene => (),
        };
    }

    Tscn {
        nodes,
        sub_resources,
    }
}
