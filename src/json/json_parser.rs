use std::collections::HashMap;

use crate::json::{json_components::Json, util::*};

pub fn parse_null(input: &str) -> Option<(Json, &str)> {
    if let Some(n_res) = get_word(input, "null") {
        Some((Json::Null, n_res))
    } else {
        None
    }
}

pub fn parse_boolean(input: &str) -> Option<(Json, &str)> {
    if let Some(b_res) = get_word(input, "true") {
        Some((Json::Boolean(true), b_res))
    } else if let Some(b_res) = get_word(input, "false") {
        Some((Json::Boolean(false), b_res))
    } else {
        None
    }
}

fn parse_hexes(input: &str) -> Option<char> {
    if let Ok(num) = u32::from_str_radix(&input[..4], 16) {
        char::from_u32(num)
    } else {
        None
    }
}

fn parse_string_until(input: &str, delim: char) -> Option<(Json, &str)> {
    if let Some(mut mat) = input.find(delim) {
        while let Some('\\') = input.chars().nth(mat-1) {
            mat += 1 + input[mat+1..].find(delim)?
        }

        // TODO: do this with a loop.
        let mut result = input[..mat].to_string();
        result = result.replace("\\\\", "\\");
        result = result.replace("\\\"", "\"");
        result = result.replace("\\/", "/");
        result = result.replace("\\n", "\n");
        result = result.replace("\\t", "\t");

        // Hex parsing. Max 4 characters.
        let mut to_replace: HashMap<String, char> = HashMap::new();
        let mut uni = result.find("\\u");
        while let Some(res) = uni {
            if let Some(c) = parse_hexes(&result[res+2..]) {
                to_replace.insert(result[res..res+6].to_string(), c);
                uni = result[res+6..].find("\\u").map(|r| r + res + 6);
            } else {
                panic!("Bad unicode")
            }
        }

        for (key, value) in to_replace {
            result = result.replace(&key, &value.to_string());
        }

        return Some((Json::String(result), &input[mat+1..]))
    }
    panic!("Bad string")
}

pub fn parse_string(input: &str) -> Option<(Json, &str)> {
    if let Some(cl) = get_char_s(input, '"') {
        parse_string_until(cl, '"')
    } else if let Some(cl) = get_char_s(input, '\'') {
        parse_string_until(cl, '\'')
    } else {
        None
    }
}

fn get_exponent(input: &str) -> Option<(&str, &str)> {
    if let Some(ex_res) = get_chars_s(input, vec!['e', 'E']) {
        if let Some(s_res) = get_char_s(ex_res.1, '-') {
            return get_number_string(s_res).map(|(_, rest)| (s_res, rest))
        } else {
            return get_number_string(input)
        }
    }
    None
}

fn get_decimals(input: &str) -> Option<(&str, &str)> {
    if let Some(d_res) = get_char_s(input, '.') {
        get_number_string(d_res)
    } else {
        None
    }
}

fn to_float(base: &str, decimals: Option<&str>) -> f64 {
    let mut base = base.to_string();
    if let Some(decs) = decimals {
        base.push('.');
        base.push_str(decs);
    }
    base.parse::<f64>().unwrap()
}

pub fn parse_number(input: &str) -> Option<(Json, &str)> {    
    if let Some(s_res) = get_char_s(input, '-') {
        if let Some((Json::Number(res_num), rest)) = parse_number(s_res) {
            Some((Json::Number(-res_num), rest))
        } else {
            panic!("Bad negative number")
        }
    } else if let Some(num_res) = get_number_string(input) {
        if let Some(exp_res) = get_exponent(num_res.1) {
            let number = to_float(num_res.0, None) * (10_f64).powf(to_float(exp_res.0, None));
            return Some((Json::Number(number), exp_res.1))
        } else if let Some(decimal_res) = get_decimals(num_res.1) {
            if let Some(exp_res) = get_exponent(decimal_res.1) {
                let number = to_float(num_res.0, Some(decimal_res.0)) * (10_f64).powf(to_float(exp_res.0, None));
                return Some((Json::Number(number), exp_res.1))
            } else {
                return Some((Json::Number(to_float(num_res.0, Some(decimal_res.0))), decimal_res.1))
            }
        } else {
            return Some((Json::Number(to_float(num_res.0, None)), num_res.1))
        }
    } else {
        None
    }
}

pub fn parse_array(input: &str) -> Option<(Json, &str)> {
    if let Some(b1_res) = get_char_s(input, '[') {
        let mut arr: Vec<Json> = Vec::new();
        let mut parse_result = parse_json(b1_res);
        while let Some(el) = parse_result {
            arr.push(el.0);
            if let Some(c_res) = get_char_s(el.1, ',') {
                parse_result = parse_json(c_res);
            } else if let Some(b2_res) = get_char_s(el.1, ']') {
                return Some((Json::Array(arr), b2_res))
            } else {
                panic!("Bad array")
            }
        }
    } 
    None
}

fn parse_key_value(input: &str) -> Option<(String, Json, &str)> {
    if let Some((Json::String(key), v_res)) = parse_string(input) {
        if let Some(d_res) = get_char_s(v_res, ':') {
            if let Some(value) = parse_json(d_res) {
                return Some((key, value.0, value.1))
            }
        }
    } 
    None
}

pub fn parse_object(input: &str) -> Option<(Json, &str)> {
    if let Some(b1_res) = get_char_s(input, '{') {
        let mut map: HashMap<String, Json> = HashMap::new();
        let mut parse_result = parse_key_value(b1_res);

        while let Some(el) = parse_result {
            map.insert(el.0, el.1);
            if let Some(c_res) = get_char_s(el.2, ',') {
                parse_result = parse_key_value(c_res);
            } else if let Some(b2_res) = get_char_s(el.2, '}') {
                return Some((Json::Object(map), b2_res))
            } else {
                panic!("bad object")
            }
        }
    } 
    None
}

pub fn parse_json(input: &str) -> Option<(Json, &str)> {
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