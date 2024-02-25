use std::{env, fs};

use json::jq_parser;

use crate::json::jq_interpreter;
use crate::json::json_print::print;

pub mod json;

fn main() {
    let mut arguments = env::args();
    if arguments.len() != 3 {
        eprintln!("There should be extra 2 arguments: a file name and a Jq string");
        return
    }
    
    let mut input: String = fs::read_to_string(arguments.nth(1).unwrap()).expect("Cannot find file");

    // let mut input = fs::read_to_string("test.json").unwrap();
    // let jq_text = ".pom.dependencies[1:4][].a".to_string();

    if let Some(res) = json::json_parser::parse_json(&mut input) {
        let jq_text = arguments.nth(0).expect("Unreachable");
        if let Some(parsed) = jq_parser::parse(jq_text.as_str()) {
            let result = jq_interpreter::interpret(vec![res], parsed);
            match result {
                Ok(res) => print!("{}", res.iter().map(print).collect::<Vec<String>>().join("\n")),
                Err(err) => eprintln!("Error with interpreting: {err}")
            }
        } else {
            eprintln!("Failed to parse the Jq string")
        }
    } else {
        eprintln!("Failed to parse the JSON file");
    }
}
