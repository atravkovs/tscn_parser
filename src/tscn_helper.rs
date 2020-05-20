use crate::str_helper::StrHelper;
use std::collections::HashMap;
use std::convert::TryInto;

use fletcher::Fletcher16;
use indexmap::IndexMap;
use regex::Regex;

use nalgebra::Vector2;

lazy_static! {
    static ref RE_VECTOR: Regex =
        Regex::new(r"^Vector2\( (-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?) \)$")
            .expect("Failed to read regex pattern");
    static ref RE_RECT: Regex =
        Regex::new(r"^Rect2\( (.+) \)$").expect("Failed to read regex pattern");
    static ref RE_VECTOR_POOL: Regex =
        Regex::new(r"^PoolVector2Array\( (.+) \)$").expect("Failed to read regex pattern");
    static ref RE_INT_POOL: Regex =
        Regex::new(r"^PoolIntArray\( (.+) \)$").expect("Failed to read regex pattern");
    static ref RE_REAL_POOL: Regex =
        Regex::new(r"^PoolRealArray\( (.+) \)$").expect("Failed to read regex pattern");
    static ref RE_SUBRES: Regex =
        Regex::new(r"^SubResource\( (\d+) \)$").expect("Failed to read regex pattern");
    static ref RE_EXTRES: Regex =
        Regex::new(r"^ExtResource\( (\d+) \)$").expect("Failed to read regex pattern");
}

#[derive(Debug, Clone)]
pub enum VarType {
    Num(isize),
    Bool(bool),
    Float(f32),
    Str(String),
    Rect2([f32; 4]),
    IntArr(Vec<isize>),
    FloatArr(Vec<f32>),
    Vector(Vector2<f32>),
    VectorArr(Vec<Vector2<f32>>),
    Map(HashMap<String, VarType>),
    SubResource(usize),
    ExtResource(usize),
    None,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Node,
    GdScene,
    SubResource,
    ExtResource,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub lhs: String,
    pub rhs: VarType,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub format: usize,
    pub load_steps: usize,
    pub path: String,
    pub name: String,
    pub rtype: String,
    pub parent: String,
    pub node_type: NodeType,
    pub instance_resource_id: usize,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            id: 0,
            format: 0,
            load_steps: 0,
            path: String::from(""),
            name: String::from(""),
            rtype: String::from(""),
            parent: String::from(""),
            instance_resource_id: 0,
            node_type: NodeType::Node,
        }
    }
}

pub struct TscnHelper();

impl TscnHelper {
    // (node_type: &str, attributes: Vec<&str>)
    pub fn parse_node(line: &str) -> (&str, &str) {
        let contents = line.trim_start_matches('[').trim_end_matches(']');

        let rtype: &str = contents.split_whitespace().next().unwrap();

        let params_str = contents.split_at(rtype.len()).1.trim();

        (rtype, params_str)
    }

    fn parse_rhs<'a>(rhs_data: &'a str) -> VarType {
        if rhs_data.check_borders('"', '"') {
            return VarType::Str(String::from(rhs_data.trim_matches('"')));
        }

        // TODO Add support for objects in array
        if rhs_data.trim() == "{" || rhs_data.split_whitespace().collect::<String>() == "[{" {
            return VarType::Map(HashMap::default());
        }

        if rhs_data == "true" || rhs_data == "false" {
            return VarType::Bool(rhs_data.parse::<bool>().unwrap());
        }

        if RE_VECTOR.is_match(rhs_data) {
            let caps = RE_VECTOR.captures(rhs_data).unwrap();

            let x = caps.get(1).unwrap().as_str();
            let y = caps.get(2).unwrap().as_str();

            return VarType::Vector(Vector2::new(
                x.parse::<f32>().unwrap(),
                y.parse::<f32>().unwrap(),
            ));
        }

        if RE_VECTOR_POOL.is_match(rhs_data) {
            let caps = RE_VECTOR_POOL.captures(rhs_data).unwrap();
            let arr_str = caps.get(1).unwrap().as_str().split(", ");
            let mut vec_arr: Vec<Vector2<f32>> = Vec::default();
            let mut previous: f32 = 0.0;
            let mut i = 0;

            for s in arr_str {
                let current = s.parse::<f32>().unwrap();

                if i % 2 == 1 {
                    vec_arr.push(Vector2::new(previous, current));
                }

                previous = current;
                i += 1;
            }

            return VarType::VectorArr(vec_arr);
        }

        if RE_RECT.is_match(rhs_data) {
            let caps = RE_RECT.captures(rhs_data).unwrap();
            let s: Vec<&str> = caps.get(1).unwrap().as_str().split(", ").collect();

            let arr = [
                s[0].parse::<f32>().unwrap(),
                s[1].parse::<f32>().unwrap(),
                s[2].parse::<f32>().unwrap(),
                s[3].parse::<f32>().unwrap(),
            ];

            return VarType::Rect2(arr);
        }

        if RE_INT_POOL.is_match(rhs_data) {
            let caps = RE_INT_POOL.captures(rhs_data).unwrap();
            let arr_str = caps.get(1).unwrap().as_str().split(", ");
            let mut int_arr: Vec<isize> = Vec::default();

            for s in arr_str {
                int_arr.push(s.parse::<isize>().unwrap());
            }

            return VarType::IntArr(int_arr);
        }

        if RE_REAL_POOL.is_match(rhs_data) {
            let caps = RE_REAL_POOL.captures(rhs_data).unwrap();
            let arr_str = caps.get(1).unwrap().as_str().split(", ");
            let mut real_arr: Vec<f32> = Vec::default();

            for s in arr_str {
                real_arr.push(s.parse::<f32>().unwrap());
            }

            return VarType::FloatArr(real_arr);
        }

        if RE_SUBRES.is_match(rhs_data) {
            let caps = RE_SUBRES.captures(rhs_data).unwrap();
            let id: usize = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();

            return VarType::SubResource(id);
        }

        if RE_EXTRES.is_match(rhs_data) {
            let caps = RE_EXTRES.captures(rhs_data).unwrap();
            let id: usize = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();

            return VarType::ExtResource(id);
        }

        if let Ok(num) = rhs_data.parse::<isize>() {
            return VarType::Num(num);
        }

        if let Ok(fl) = rhs_data.parse::<f32>() {
            return VarType::Float(fl);
        }

        VarType::None
    }

    fn parse_eq<'a>(cmd_data: [&'a str; 2]) -> Command {
        let lhs_data = cmd_data[0].trim();
        let rhs_data = cmd_data[1].trim();

        let rhs = Self::parse_rhs(rhs_data);

        Command {
            lhs: lhs_data.to_string(),
            rhs,
        }
    }

    pub fn parse_command<'a>(line: &'a str) -> Option<Command> {
        let cmd_data: Vec<&str> = line.split("=").collect();

        if cmd_data.len() != 2 {
            return None;
        }

        Some(Self::parse_eq([cmd_data[0], cmd_data[1]]))
    }

    pub fn parse_obj<'a>(line: &'a str) -> Option<Command> {
        let cmd_data: Vec<&str> = line.split(":").collect();

        if cmd_data.len() != 2 {
            return None;
        }

        let lhs = cmd_data[0].trim_matches('"');

        Some(Self::parse_eq([
            lhs,
            cmd_data[1].trim().trim_end_matches(','),
        ]))
    }

    pub fn get_node<'a>(node_type: &'a str, attributes_str: &'a str) -> Node {
        let mut node = Node::default();

        match node_type {
            "gd_scene" => node.node_type = NodeType::GdScene,
            "sub_resource" => node.node_type = NodeType::SubResource,
            "ext_resource" => node.node_type = NodeType::ExtResource,
            _ => (),
        };

        let attributes = Self::split_attributes(attributes_str);
        for attribute in attributes {
            let attr_name = attribute.0.as_str();

            match attr_name {
                "id" => {
                    node.id = if let VarType::Num(id) = attribute.1 {
                        id.try_into().unwrap()
                    } else {
                        0
                    }
                }
                "name" => {
                    node.name = if let VarType::Str(n) = attribute.1 {
                        String::from(n)
                    } else {
                        "".to_string()
                    }
                }
                "type" => {
                    node.rtype = if let VarType::Str(rt) = attribute.1 {
                        String::from(rt)
                    } else {
                        "".to_string()
                    }
                }
                "parent" => {
                    node.parent = if let VarType::Str(p) = attribute.1 {
                        String::from(p)
                    } else {
                        "".to_string()
                    }
                }
                "instance" => {
                    node.instance_resource_id = if let VarType::ExtResource(r) = attribute.1 {
                        r
                    } else {
                        0
                    }
                }
                "path" => {
                    node.path = if let VarType::Str(path) = attribute.1 {
                        String::from(path)
                    } else {
                        "".to_string()
                    }
                }
                "load_steps" => {
                    node.load_steps = if let VarType::Num(ls) = attribute.1 {
                        ls.try_into().unwrap()
                    } else {
                        0
                    }
                }
                "format" => {
                    node.format = if let VarType::Num(f) = attribute.1 {
                        f.try_into().unwrap()
                    } else {
                        0
                    }
                }
                wtf => unimplemented!("{}", wtf),
            }
        }

        node
    }

    pub fn get_path_hash(ctx: &IndexMap<String, usize>) -> u16 {
        let mut checksum = Fletcher16::new();
        let mut path = "/root/root".to_string();

        for key in ctx.keys() {
            if key == &"." {
                continue;
            }

            path.push_str(&format!("/{}", key));
        }

        checksum.update(&path.as_bytes());

        checksum.value()
    }

    fn split_attributes<'a>(cmd_line: &'a str) -> Vec<(String, VarType)> {
        let mut commands: Vec<(String, VarType)> = Vec::new();

        let mut is_lhs = true;
        let mut isq_opened = false;
        let mut isb_opened = false;
        let mut lhs = String::from("");
        let mut rhs = String::from("");
        let mut previous = ' ';

        let line = cmd_line.trim();

        for (i, ch) in line.chars().enumerate() {
            if ch == '=' {
                is_lhs = false;
                continue;
            }

            if ch == ' ' {
                previous = ' ';
                continue;
            }

            if ch == '"' {
                isq_opened = !isq_opened;
            }

            if ch == '(' {
                isb_opened = true;
            }

            if ch == ')' {
                isb_opened = false;
            }

            if i == line.len() - 1 {
                rhs.push(ch);
                commands.push((lhs, Self::parse_rhs(&rhs)));
                break;
            }

            if previous == ' '
                && ch.is_alphabetic()
                && is_lhs == false
                && lhs != ""
                && rhs != ""
                && !isq_opened
                && !isb_opened
            {
                commands.push((lhs, Self::parse_rhs(&rhs)));
                lhs = String::from("");
                lhs.push(ch);
                rhs = String::from("");
                is_lhs = true;
                previous = ' ';
                continue;
            }

            if is_lhs {
                lhs = format!("{}{}", lhs, ch);
            } else {
                rhs = format!("{}{}", rhs, ch);
            }

            previous = ch;
        }

        commands
    }
}
