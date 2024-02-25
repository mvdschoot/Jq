use std::collections::HashMap;

use crate::json::json_components::Json;

fn print_null() -> String {
    return "null".to_owned();
}

fn print_boolean(b: &bool) -> String {
    match b {
        true => "true".to_owned(),
        false => "false".to_owned()
    }
}

fn print_number(n: &f64) -> String {
    n.to_string()
}

fn print_string(s: &String) -> String {
    let mut r = "\"".to_owned();
    r.push_str(s.as_str());
    r.push('"');
    r
}

fn print_array(a: &Vec<Json>) -> String {
    let mut r = "[".to_string();
    for (i, el) in a.into_iter().enumerate() {
        r.extend(print(el).chars());
        
        if i != a.len()-1 {
            r.extend(",".to_string().chars())
        }
    }
    r.extend("]".to_string().chars());
    r
}

fn print_object(m: &HashMap<String, Json>) -> String {
    let mut r = "{".to_owned();
    for (i, (key, value)) in m.into_iter().enumerate() {
        r.extend(print_string(key).chars());
        r.extend(":".to_string().chars());
        r.extend(print(value).chars());
        
        if i != m.len()-1 {
            r.extend(", ".to_string().chars())
        }
    }
    r.extend("}".to_string().chars());
    r
}

pub fn print(j: &Json) -> String {
    match j {
        Json::Null => print_null(),
        Json::Boolean(b) => print_boolean(b),
        Json::Number(j) => print_number(j),
        Json::String(j) => print_string(j),
        Json::Array(j) => print_array(j),
        Json::Object(j) => print_object(j),
    }
}