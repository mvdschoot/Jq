use std::collections::HashMap;

use crate::json::json_components::Json;

fn skip_stuff(input: &mut String) {
    let to_skip = vec!['\n', '\t', ' '];

    for (i, char) in input.chars().enumerate()  {
        if !to_skip.contains(&char) {
            trim(input, i);
            return
        }
    }
}

fn get_word(input: &mut String, word: &str) -> Option<usize> {
    skip_stuff(input);
    if input.len() == 0 {
        return None
    }

    let zipped = word.chars().zip(input.chars());
    for (w, i) in zipped {
        if w != i {
            return None
        }
    }
    return Some(word.len())
}

fn get_char(input: &mut String, c: char) -> Option<usize> {
    skip_stuff(input);
    if let Some(ch) = input.chars().nth(0) {
        if c == ch {
            return Some(1);
        }
    }
    None
}

fn get_chars(input: &mut String, cs: Vec<char>) -> Option<char> {
    skip_stuff(input);
    input.chars().next().and_then(|ch| {
        if cs.contains(&ch) {
            Some(ch)
        } else {
            None
        }
    })
}

fn trim(input: &mut String, n: usize) {
    let len = input.len().min(n);
    input.drain(..len);
}

pub fn parse_null(input: &mut String) -> Option<Json> {
    if let Some(l) = get_word(input, "null") {
        trim(input, l);
        Some(Json::Null)
    } else {
        None
    }
}

pub fn parse_boolean(input: &mut String) -> Option<Json> {
    if let Some(l) = get_word(input, "true") {
        trim(input, l);
        Some(Json::Boolean(true))
    } else if let Some(l) = get_word(input, "false") {
        trim(input, l);
        Some(Json::Boolean(false))
    } else {
        None
    }
}

fn parse_hexes(input: &String, loc: usize) -> Option<char> {
    if let Ok(num) = u32::from_str_radix(&input[loc..loc+4], 16) {
        char::from_u32(num)
    } else {
        None
    }
}

fn parse_string_until(input: &mut String, delim: char) -> Option<Json> {
    trim(input, 1);
    if let Some(mut mat) = input.find(delim) {
        while let Some('\\') = input.chars().nth(mat-1) {
            mat += 1 + input[(mat+1)..].find(delim).unwrap();
        }

        let mut result = input[..mat].to_string();
        result = result.replace("\\\\", "\\");
        result = result.replace("\\\"", "\"");
        result = result.replace("\\/", "/");
        result = result.replace("\\n", "\n");
        result = result.replace("\\t", "\t");

        let mut to_replace: HashMap<String, char> = HashMap::new();
        let mut uni = result.find("\\u");
        while let Some(res) = uni {
            if let Some(c) = parse_hexes(&result, res+2) {
                to_replace.insert(result[res..res+6].to_string(), c);
                uni = result[res+6..].find("\\u").map(|r| r + res + 6);
            } else {
                panic!("Bad unicode")
            }
        }

        for (key, value) in to_replace {
            result = result.replace(&key, &value.to_string());
        }

        trim(input, mat+1);
        return Some(Json::String(result))
    }
    panic!("Bad string")
}

pub fn parse_string(input: &mut String) -> Option<Json> {
    if let Some(_cl) = get_char(input, '"') {
        parse_string_until(input, '"')
    } else if let Some(_cl) = get_char(input, '\'') {
        parse_string_until(input, '\'')
    } else {
        None
    }
}

fn get_number(input: &mut String) -> Option<String> {
    let mut v = String::new();
    for c in input.chars() {
        if c.is_numeric() {
            v.push(c);
        } else {
            break;
        }
    }

    let len = v.len();
    if len == 0 {
        None
    } else {
        trim(input, len);
        Some(v)
    }
}

fn get_exponent(input: &mut String) -> Option<String> {
    if let Some(_c1) = get_chars(input, vec!['e', 'E']) {
        trim(input, 1);
        if let Some(_sign) = get_char(input, '-') {
            trim(input, 1);
            if let Some(digit) = get_number(input) {
                Some(String::from("-") + &digit)
            } else {
                panic!("Bad exponent")
            }
        } else {
            get_number(input)
        }
    } else {
        None
    }
}

fn get_decimals(input: &mut String) -> Option<String> {
    if let Some(_c1) = get_char(input, '.') {
        trim(input, 1);
        get_number(input)
    } else {
        None
    }
}

fn to_float(base: String, decimals: Option<String>) -> f64 {
    let mut base = base;
    if let Some(decs) = decimals {
        base.push('.');
        base.push_str(decs.as_str());
    }
    base.parse::<f64>().unwrap()
}

pub fn parse_number(input: &mut String) -> Option<Json> {    
    if let Some(_c1) = get_char(input, '-') {
        trim(input, 1);
        if let Some(Json::Number(res_num)) = parse_number(input) {
            Some(Json::Number(-res_num))
        } else {
            panic!("Bad negative number")
        }
    } else if let Some(num) = get_number(input) {
        if let Some(exp) = get_exponent(input) {
            Some(Json::Number(to_float(num, None) * (10_f64).powf(to_float(exp, None))))
        } else if let Some(decimals) = get_decimals(input) {
            if let Some(exp) = get_exponent(input) {
                Some(Json::Number(to_float(num, Some(decimals)) * (10_f64).powf(to_float(exp, None))))
            } else {
                Some(Json::Number(to_float(num, Some(decimals))))
            }
        } else {
            Some(Json::Number(to_float(num, None)))
        }
    } else {
        None
    }
}

pub fn parse_array(input: &mut String) -> Option<Json> {
    if let Some(_b) = get_char(input, '[') {
        trim(input, 1);
        let mut arr: Vec<Json> = Vec::new();
        let mut parse_result: Option<Json> = parse_json(input);
        while let Some(ref el) = parse_result {
            arr.push(el.clone());
            if let Some(_p) = get_char(input, ',') {
                trim(input, 1);
                parse_result = parse_json(input);
            } else if let Some(_p) = get_char(input, ']') {
                trim(input, 1);
                return Some(Json::Array(arr))
            } else {
                panic!("Bad array")
            }
        }
    } 
    None
}

fn parse_key_value(input: &mut String) -> Option<(String, Json)> {
    if let Some(Json::String(key)) = parse_string(input) {
        if let Some(_delim) = get_char(input, ':') {
            trim(input, 1);
            if let Some(value) = parse_json(input) {
                return Some((key, value))
            }
        }
    } 
    None
}

pub fn parse_object(input: &mut String) -> Option<Json> {
    if let Some(_b) = get_char(input, '{') {
        trim(input, 1);
        let mut map: HashMap<String, Json> = HashMap::new();
        let mut parse_result = parse_key_value(input);

        while let Some(ref el) = parse_result {
            map.insert(el.0.clone(), el.1.clone());
            if let Some(_p) = get_char(input, ',') {
                trim(input, 1);
                parse_result = parse_key_value(input);
            } else if let Some(_p) = get_char(input, '}') {
                trim(input, 1);
                return Some(Json::Object(map))
            } else {
                panic!("bad object")
            }
        }
    } 
    None
}

pub fn parse_json(input: &mut String) -> Option<Json> {
    if let Some(json) = parse_null(input) {
        return Some(json);
    }
    if let Some(json) = parse_number(input) {
        return Some(json);
    }
    if let Some(json) = parse_boolean(input) {
        return Some(json);
    }
    if let Some(json) = parse_string(input) {
        return Some(json);
    }
    if let Some(json) = parse_array(input) {
        return Some(json);
    }
    if let Some(json) = parse_object(input) {
        return Some(json);
    }
    None
}