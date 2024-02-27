use std::collections::HashMap;

use colored::{ColoredString, Colorize};

use crate::json::json_components::Json;

type Line = Vec<ColoredString>;

fn print_null() -> ColoredString {
    return "null".cyan()
}

fn print_boolean(b: &bool) -> ColoredString {
    match b {
        true => "true",
        false => "false"
    }.truecolor(243, 223, 162)
}

fn print_number(n: &f64) -> ColoredString {
    n.to_string().bright_red()
}

fn print_string(s: &String) -> String {
    let mut r = "\"".to_owned();
    r.push_str(s.as_str());
    r.push('"');
    r
}

fn print_string_c(s: &String) -> ColoredString {
    print_string(s).green()
}

fn print_array(a: &Vec<Json>) -> Vec<Line> {
    let mut r = vec![vec!["[".purple()]];
    for (i, el) in a.into_iter().enumerate() {
        let mut lines = print(el);
        lines.iter_mut().for_each(|p| p.insert(0, "\t".normal()));
        
        if i != a.len()-1 {
            if let Some(last) = lines.last_mut() {
                last.push(", ".normal());
            }
        }
        r.extend(lines);
    }
    r.push(vec!["]".purple()]);
    r
}

fn print_object(m: &HashMap<String, Json>) -> Vec<Line> {
    // Sort by key name
    let mut m = Vec::from_iter(m.into_iter());
    m.sort_by_key(|&(k, _)| k);

    let mut r = vec![vec!["{".purple()]];
    for (i, (key, value)) in m.clone().into_iter().enumerate() {
        let mut line = vec!["\t".normal(), print_string(key).truecolor(145, 120, 93)];
        line.push(": ".normal());
        let mut printed_value = print(value);

        if let Some(first) = printed_value.first() {
            line.extend(first.clone());
            r.push(line);
            if let Some(rest) = printed_value.get_mut(1..) {
                rest.iter_mut().for_each(|l| l.insert(0, "\t".normal()));
                r.extend(rest.to_vec());
            }
        }
        
        if i != m.len()-1 {
            if let Some(last) = r.last_mut() {
                last.push(", ".normal());
            }
        }
    }
    r.push(vec!["}".purple()]);
    r
}

pub fn print(j: &Json) -> Vec<Line> {
    match j {
        Json::Null => vec![vec![print_null()]],
        Json::Boolean(b) => vec![vec![print_boolean(b)]],
        Json::Number(j) => vec![vec![print_number(j)]],
        Json::String(j) => vec![vec![print_string_c(j)]],
        Json::Array(j) => print_array(j),
        Json::Object(j) => print_object(j),
    }
}