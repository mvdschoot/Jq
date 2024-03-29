use std::path::PathBuf;
use std::{env, fs};

use colored::ColoredString;
use json::jq_parser;

use crate::json::jq_interpreter;
use crate::json::json_print::print;

pub mod json;

fn main() {
    let mut arguments = env::args();
    if arguments.len() < 2 || arguments.len() > 3 {
        eprintln!("There should be extra 2 arguments: a file name and a Jq string");
        return
    }

    let mut file_location: String = "./".to_string();
    if let Ok(cwd) = env::current_dir() {
        file_location = cwd.join(PathBuf::from(arguments.nth(1).unwrap())).to_str().unwrap().to_string();
    }
    let mut input: String = fs::read_to_string(file_location).expect("Cannot find file");

    // let mut input = fs::read_to_string("test.json").unwrap();
    // let jq_text = ".pom.dependencies[].a | . * 3".to_string();

    if let Some((res, leftover)) = json::json_parser::parse_json(&mut input) {
        if leftover == "" {
            let jq_text = arguments.nth(0).unwrap_or(".".to_string());
            if let Some(parsed) = jq_parser::parse(jq_text.as_str()) {
                let result = jq_interpreter::interpret(vec![res], parsed);
                match result {
                    Ok(res) => {
                        let to_print = res.iter().map(print).collect::<Vec<Vec<Vec<ColoredString>>>>();
                        for l in to_print {
                            for l2 in l {
                                for l3 in l2 {
                                    print!("{}", l3);
                                }
                                print!("\n")
                            }
                        }
                        print!("\n")
                    },
                    Err(err) => eprintln!("Error with interpreting: {err}")
                }
            } else {
                eprintln!("Failed to parse the Jq string")
            }
        } else {
            eprintln!("Failed to parse json, leftover: {leftover}")
        }
    } else {
        eprintln!("Failed to parse the JSON file");
    }
}
