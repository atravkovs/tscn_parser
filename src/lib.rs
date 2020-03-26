#[macro_use]
extern crate lazy_static;

use indexmap::IndexMap;
use std::collections::HashMap;

use str_helper::StrHelper;
use tscn_helper::{Command, NodeType, TscnHelper, VarType};

pub mod str_helper;
pub mod tscn_helper;

#[derive(Debug, Clone)]
pub struct SubResourceEntry<'a> {
    pub rtype: &'a str,
    pub properties: HashMap<&'a str, VarType<'a>>,
}

#[derive(Debug, Clone)]
pub struct NodeEntry<'a> {
    pub uuid: u16,
    pub level: usize,
    pub name: &'a str,
    pub rtype: &'a str,
    pub parent_id: usize,
    pub properties: HashMap<&'a str, VarType<'a>>,
    pub childrens: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Tscn<'a> {
    pub nodes: HashMap<usize, NodeEntry<'a>>,
    pub sub_resources: HashMap<usize, SubResourceEntry<'a>>,
}

impl<'a> Default for NodeEntry<'a> {
    fn default() -> Self {
        NodeEntry {
            uuid: 0,
            name: "",
            level: 0,
            rtype: "",
            parent_id: 0,
            childrens: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl<'a> SubResourceEntry<'a> {
    fn new(rtype: &'a str) -> Self {
        SubResourceEntry {
            rtype,
            properties: HashMap::new(),
        }
    }
}

pub fn parse_tscn<'a>(tscn: &'a str) -> Tscn<'a> {
    let mut context: Option<NodeType<'a>> = None;
    let mut ctx: IndexMap<&str, usize> = IndexMap::new();

    let mut nodes: HashMap<usize, NodeEntry<'a>> = HashMap::new();
    let mut sub_resources = HashMap::new();

    let mut node_id: usize = 0;

    for line in tscn.lines() {
        if line.is_empty() {
            continue;
        }

        if line.check_borders('[', ']') {
            let (node_type, attributes) = TscnHelper::parse_node(line);
            context = TscnHelper::get_context(node_type, attributes);

            if context.is_none() {
                continue;
            }

            match context.unwrap() {
                NodeType::SubResource(sub_resource) => {
                    sub_resources
                        .insert(sub_resource.id, SubResourceEntry::new(sub_resource.rtype));
                }
                NodeType::Node(node) => {
                    let entry: NodeEntry = match node.parent {
                        "" => {
                            ctx.insert(".", node_id);

                            NodeEntry {
                                name: node.name,
                                rtype: node.rtype,
                                ..NodeEntry::default()
                            }
                        }
                        parent => {
                            let clonned_ctx = ctx.clone();
                            let (level, _, parent_id) = clonned_ctx.get_full(parent).unwrap();

                            let last_id = ctx.len() - 1;
                            if level != last_id {
                                for _ in level..last_id {
                                    ctx.pop();
                                }
                            }

                            ctx.insert(node.name, node_id);
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
                        }
                    };

                    nodes.insert(node_id, entry);
                    node_id += 1;
                }
            }

            continue;
        }

        if context.is_none() {
            continue;
        }

        let command: Command = TscnHelper::parse_command(line);

        match context.unwrap() {
            NodeType::Node(_) => {
                let (_, id) = ctx.get_index(ctx.len() - 1).unwrap();
                nodes
                    .get_mut(id)
                    .unwrap()
                    .properties
                    .insert(command.lhs, command.rhs);
            }
            NodeType::SubResource(sub_resource) => {
                sub_resources
                    .get_mut(&sub_resource.id)
                    .unwrap()
                    .properties
                    .insert(command.lhs, command.rhs);
            }
        };
    }

    Tscn {
        nodes,
        sub_resources,
    }
}
