use glob::{glob, GlobError};
use std::fs;

use ::tscn_parser::Loader;

fn main() {
    println!("Reading available scenes...");
    let scenes = parse("../game/client/common-assets/Scenes/*.tscn").expect("Error reading path");

    for scene in scenes {
        let tscn = Loader::new().parse_tscn(&scene);
        println!("SubResources:\n--------------------------------------------------");
        for (id, res) in &tscn.sub_resources {
            println!("{:?}: {:?}", id, res);
        }

        println!("Nodes:\n---------------------------------------------------------");
        for (id, node) in &tscn.nodes {
            println!("{:?}: {:?}", id, node);
        }

        println!("Resource:\n---------------------------------------------------------");
        println!("{:?}", &tscn.resource);
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
