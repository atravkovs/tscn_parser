#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs;

use glob::{glob, GlobError};
use indexmap::IndexMap;
use regex::Regex;

use specs_physics::nphysics::math::Vector;

lazy_static! {
    static ref RE_VECTOR: Regex =
        Regex::new(r"^Vector2\( (-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?) \)$")
            .expect("Failed to read regex pattern");
    static ref RE_SUBRES: Regex =
        Regex::new(r"^SubResource\( (\d+) \)$").expect("Failed to read regex pattern");
    static ref RE_EXTRES: Regex =
        Regex::new(r"^ExtResource\( (\d+) \)$").expect("Failed to read regex pattern");
}

#[derive(Debug, Clone, Copy)]
struct SubResource<'a> {
    id: usize,
    rtype: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct Node<'a> {
    name: &'a str,
    rtype: &'a str,
    parent: &'a str,
}

#[derive(Debug, Clone, Copy)]
enum NodeType<'a> {
    Node(Node<'a>),
    SubResource(SubResource<'a>),
}

#[derive(Debug, Clone, Copy)]
enum VarType<'a> {
    Num(usize),
    Bool(bool),
    Float(f32),
    Str(&'a str),
    Vector(Vector<f32>),
    SubResource(usize),
    ExtResource(usize),
    None(&'a str),
}

#[derive(Debug, Clone, Copy)]
struct Command<'a> {
    lhs: &'a str,
    rhs: VarType<'a>,
}

#[derive(Debug, Clone)]
struct SubResourceEntry<'a> {
    rtype: &'a str,
    properties: HashMap<&'a str, VarType<'a>>,
}

#[derive(Debug, Clone)]
struct NodeEntry<'a> {
    level: usize,
    name: &'a str,
    rtype: &'a str,
    parent_id: usize,
    properties: HashMap<&'a str, VarType<'a>>,
    childrens: Vec<usize>,
}

#[derive(Debug, Clone)]
struct Tscn<'a> {
    nodes: HashMap<usize, NodeEntry<'a>>,
    sub_resources: HashMap<usize, SubResourceEntry<'a>>,
}

impl<'a> Default for NodeEntry<'a> {
    fn default() -> Self {
        NodeEntry {
            name: "",
            level: 0,
            rtype: "",
            parent_id: 0,
            childrens: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl<'a> NodeEntry<'a> {
    fn new(name: &'a str, rtype: &'a str, level: usize, parent_id: usize) -> Self {
        NodeEntry {
            name,
            rtype,
            level,
            parent_id,
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

fn parse_tscn<'a>(tscn: &'a str) -> Tscn<'a> {
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

fn parse(pattern: &str) -> Result<Vec<String>, GlobError> {
    let mut result: Vec<String> = Vec::new();

    for entry in glob(pattern).expect("Failed to read glob pattern") {
        let path = entry?;
        result.push(fs::read_to_string(path).expect("Something went wrong reading the file"));
    }

    Ok(result)
}

fn main() {
    println!("Reading available scenes...");
    let scenes = parse("../client/Scenes/*.tscn").expect("Error reading path");

    for scene in scenes {
        let tscn = parse_tscn(&scene);
        println!("SubResources:\n--------------------------------------------------");
        for (id, res) in &tscn.sub_resources {
            println!("{:?}: {:?}", id, res);
        }

        println!("Nodes:\n---------------------------------------------------------");
        for (id, node) in &tscn.nodes {
            println!("{:?}: {:?}", id, node);
        }
    }
}

trait StrHelpers {
    fn check_borders(&self, start_char: char, end_char: char) -> bool;
}

struct TscnHelper();

impl StrHelpers for str {
    #[inline]
    fn check_borders(&self, start_char: char, end_char: char) -> bool {
        let mut chars = self.chars();
        chars.nth(0).unwrap() == start_char && chars.rev().nth(0).unwrap() == end_char
    }
}

impl TscnHelper {
    #[inline]
    // (node_type: &str, attributes: Vec<&str>)
    fn parse_node(line: &str) -> (&str, Vec<&str>) {
        let pieces: Vec<&str> = line
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split_whitespace()
            .collect();

        (pieces[0], pieces[1..].to_vec())
    }

    fn parse_command<'a>(line: &'a str) -> Command<'a> {
        let cmd_data: Vec<&str> = line.split("=").collect();

        let lhs = cmd_data[0].trim();
        let rhs_data = cmd_data[1].trim();

        let rhs: VarType = if rhs_data.check_borders('"', '"') {
            VarType::Str(rhs_data.trim_matches('"'))
        } else if rhs_data == "true" || rhs_data == "false" {
            VarType::Bool(rhs_data.parse::<bool>().unwrap())
        } else if RE_VECTOR.is_match(rhs_data) {
            let caps = RE_VECTOR.captures(rhs_data).unwrap();

            let x = caps.get(1).unwrap().as_str();
            let y = caps.get(2).unwrap().as_str();

            VarType::Vector(Vector::new(
                x.parse::<f32>().unwrap(),
                y.parse::<f32>().unwrap(),
            ))
        } else if RE_SUBRES.is_match(rhs_data) {
            let caps = RE_SUBRES.captures(rhs_data).unwrap();
            let id: usize = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();

            VarType::SubResource(id)
        } else if RE_EXTRES.is_match(rhs_data) {
            let caps = RE_EXTRES.captures(rhs_data).unwrap();
            let id: usize = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();

            VarType::ExtResource(id)
        } else if let Ok(num) = rhs_data.parse::<usize>() {
            VarType::Num(num)
        } else if let Ok(fl) = rhs_data.parse::<f32>() {
            VarType::Float(fl)
        } else {
            VarType::None(rhs_data)
        };

        Command { lhs, rhs }
    }

    fn get_context<'a>(node_type: &'a str, attributes: Vec<&'a str>) -> Option<NodeType<'a>> {
        match node_type {
            "sub_resource" => Some(NodeType::SubResource(Self::get_sub_resource(attributes))),
            "node" => Some(NodeType::Node(Self::get_node(attributes))),
            _ => None,
        }
    }

    fn get_node<'a>(attributes: Vec<&'a str>) -> Node<'a> {
        let mut name = "";
        let mut rtype = "";
        let mut parent = "";

        for attribute_encoded in attributes {
            let attr_data: Vec<&str> = attribute_encoded.split("=").collect();

            let attr_name = attr_data[0];
            let attr_value = attr_data[1].trim_matches('"');

            match attr_name {
                "name" => name = attr_value,
                "type" => rtype = attr_value,
                "parent" => parent = attr_value,
                _ => (),
            }
        }

        Node {
            name,
            rtype,
            parent,
        }
    }

    fn get_sub_resource<'a>(attributes: Vec<&'a str>) -> SubResource<'a> {
        let mut id: usize = 0;
        let mut rtype = "";

        for attribute_encoded in attributes {
            let attr_data: Vec<&str> = attribute_encoded.split("=").collect();

            let attr_name = attr_data[0];
            let attr_value = attr_data[1];

            match attr_name {
                "id" => {
                    id = attr_value.parse().unwrap();
                }
                "type" => {
                    rtype = attr_value.trim_matches('"');
                }
                _ => (),
            }
        }

        SubResource { id, rtype }
    }
}
