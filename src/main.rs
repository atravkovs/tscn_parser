use glob::{glob, GlobError};
use std::fs;

use ::tscn_parser::parse_tscn;

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

fn parse(pattern: &str) -> Result<Vec<String>, GlobError> {
    let mut result: Vec<String> = Vec::new();

    for entry in glob(pattern).expect("Failed to read glob pattern") {
        let path = entry?;
        result.push(fs::read_to_string(path).expect("Something went wrong reading the file"));
    }

    Ok(result)
}