use crate::str_helper::StrHelper;
use regex::Regex;

use nalgebra::Vector2;

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
pub enum VarType<'a> {
    Num(usize),
    Bool(bool),
    Float(f32),
    Str(&'a str),
    Vector(Vector2<f32>),
    SubResource(usize),
    ExtResource(usize),
    None(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    pub lhs: &'a str,
    pub rhs: VarType<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct SubResource<'a> {
    pub id: usize,
    pub rtype: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    pub name: &'a str,
    pub rtype: &'a str,
    pub parent: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeType<'a> {
    Node(Node<'a>),
    SubResource(SubResource<'a>),
}

pub struct TscnHelper();

impl TscnHelper {
    #[inline]
    // (node_type: &str, attributes: Vec<&str>)
    pub fn parse_node(line: &str) -> (&str, Vec<&str>) {
        let pieces: Vec<&str> = line
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split_whitespace()
            .collect();

        (pieces[0], pieces[1..].to_vec())
    }

    pub fn parse_command<'a>(line: &'a str) -> Command<'a> {
        let cmd_data: Vec<&str> = line.split("=").collect();

        let lhs = cmd_data[0].trim();

        if cmd_data.len() < 2 || lhs == "__meta__" {
            return Command {
                lhs: "",
                rhs: VarType::None(""),
            };
        }

        let rhs_data = cmd_data[1].trim();

        let rhs: VarType = if rhs_data.check_borders('"', '"') {
            VarType::Str(rhs_data.trim_matches('"'))
        } else if rhs_data == "true" || rhs_data == "false" {
            VarType::Bool(rhs_data.parse::<bool>().unwrap())
        } else if RE_VECTOR.is_match(rhs_data) {
            let caps = RE_VECTOR.captures(rhs_data).unwrap();

            let x = caps.get(1).unwrap().as_str();
            let y = caps.get(2).unwrap().as_str();

            VarType::Vector(Vector2::new(
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

    pub fn get_context<'a>(node_type: &'a str, attributes: Vec<&'a str>) -> Option<NodeType<'a>> {
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
