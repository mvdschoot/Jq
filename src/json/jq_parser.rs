use std::collections::HashMap;

use super::{jq_components::Jq, json_parser, util::*};

fn parse_parenthesis(input: &str) -> Option<(Jq, &str)> {
    if let Some(rest) = get_char_s(input, '(') {
        if let Some(loc2) = rest.find(')') {
            if let Some(content) = parse(&rest[..loc2]) {
                return Some((Jq::Parenthesis(Box::new(content)), &rest[loc2+1..]))
            } else {
                panic!("No bad parenthesis content")
            }
        } else {
            panic!("No closing parenthesis")
        }
    }
    None
}

fn avoid_parenthesis(input: &str, to_find: char) -> Option<usize> {
    // input.find(to_find)
    let mut depths = (0, 0);
    for (i, c) in input.chars().enumerate() {
        if c == to_find && depths.0 == 0 && depths.1 == 0 {
            return Some(i)
        }
        if c == '(' {
            depths.0 += 1;
        }
        if c == '[' {
            depths.1 += 1;
        }
        if c == ')' && depths.0 > 0 {
            depths.0 -= 1;
        } else if c == ')' {
            return None
        }
        if c == ']' && depths.1 > 0 {
            depths.1 -= 1;
        } else if c == ']' {
            return None
        }
    }
    None
}

fn parse_pipe(input: &str) -> Option<(Jq, &str)> {
    if let Some(p_loc) = avoid_parenthesis(input, '|') {
        let left = parse(&input[..p_loc]).unwrap_or_else(|| Jq::Input);

        match parse(&input[p_loc+1..]) {
            Some(a) => Some((Jq::Pipe(Box::new(left), Box::new(a)), "")),
            None => panic!("Bad pipe right side"),
        }
    } else {
        None
    }
}

fn parse_comma(input: &str) -> Option<(Jq, &str)> {
    if let Some(c_loc) = avoid_parenthesis(input, ',') {
        let left = match parse(&input[..c_loc]) {
            Some(l) => l,
            None => panic!("Bad comma left side")
        };

        match parse(&input[c_loc+1..]) {
            Some(a) => Some((Jq::Comma(Box::new(left), Box::new(a)), "")),
            None => panic!("Bad comma right side"),
        }
    } else {
        None
    }
}

fn parse_optional(input: &str) -> Option<(Jq, &str)> {
    if let Some(obj_res) = parse_id_chain(input) {
        if let Some(q_res) = get_char_s(obj_res.1, '?') {
            return Some((Jq::Optional(Box::new(obj_res.0)), q_res))
        }
    }
    return None
}

fn parse_iterator(input: &str) -> Option<(Jq, &str)> {
    if let Some(dot_res) = get_char_s(&input, '.') {
        if let Some(b1_res) = get_char_s(dot_res, '[') {
            if let Some(b2_res) = get_char_s(b1_res, ']') {       
                return Some((Jq::Iterator, b2_res));
            }
        }
    }
    None
}

fn parse_slice(input: &str) -> Option<(Jq, &str)> {
    if let Some(a) = get_number(input) {
        if let Some(s_res) = get_char_s(a.1, ':') {
            if let Some(b) = get_number(s_res) {
                return Some((Jq::Slice(a.0, b.0), b.1))
            }
        }
    }
    None
}

fn is_letter(c: char) -> bool{
    (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_'
}

fn is_number(c: char) -> bool{
    c >= '1' && c <= '9'
}

fn parse_object_id(input: &str) -> Option<(Jq, &str)> {
    if skip_stuff(input).len() == 0 {
        return None
    }

    let mut any_char = get_any_char_s(input);
    let mut i = 0;
    while let Some((c, c_res)) = any_char {
        if i == 0 && !is_letter(c) {
            return None
        }

        if !is_letter(c) && !is_number(c) {
            return Some((Jq::Id(input[..i].to_string()), &input[i..]))
        }

        i += 1;
        any_char = get_any_char(c_res);
    }
    Some((Jq::Id(input.to_string()), ""))
}

fn parse_id_chain(input: &str) -> Option<(Jq, &str)> {
    if let Some(dot_res) = get_char_s(&input, '.') {
        let mut chain: Vec<Jq> = Vec::new();
        let mut id = parse_array(dot_res).or_else(|| parse_object_id(dot_res));
        let mut last: &str = input;
        while let Some(id_res) = &id {
            chain.push(id_res.0.clone());
            last = id_res.1;
            id = parse_array(id_res.1).or_else(|| {
                if let Some('.') = last.chars().next() {
                    return parse_object_id(&last[1..])
                }
                None
            });
        }

        return if chain.len() == 0 {
            Some((Jq::Identity, dot_res))
        } else {
            Some((Jq::IdChain(chain), last))
        }
    }
    None
}

fn parse_identity(input: &str) -> Option<(Jq, &str)> {
    if let Some(res) = get_char_s(&input, '.') {
        Some((Jq::Identity, res))
    } else {
        None
    }
}

fn parse_recursive(input: &str) -> Option<(Jq, &str)> {
    if let Some(p1_res) = get_char_s(&input, '.') {
        if let Some(p2_res) = get_char_s(p1_res, '.') {
            return Some((Jq::Recursive, p2_res))
        } 
    }
    None
}

fn parse_null(input: &str) -> Option<(Jq, &str)> {
    json_parser::parse_null(input)
            .map(|(json, res)| {(Jq::from(json), res)})
}

fn parse_boolean(input: &str) -> Option<(Jq, &str)> {
    json_parser::parse_boolean(input)
            .map(|(json, res)| {(Jq::from(json), res)})
}

fn parse_number(input: &str) -> Option<(Jq, &str)> {
    json_parser::parse_number(input)
            .map(|(json, res)| {(Jq::from(json), res)})
}

fn parse_string(input: &str) -> Option<(Jq, &str)> {
    json_parser::parse_string(input)
            .map(|(json, res)| {(Jq::from(json), res)})
}

fn unpack_comma_to_array(input: Jq) -> Vec<Jq> {
    match input {
        Jq::Comma(a, b) => {
            let mut res = vec![*a];
            res.append(unpack_comma_to_array(*b).as_mut());
            return res
        },
        other => vec![other]
    }
}

fn parse_array(input: &str) -> Option<(Jq, &str)> {
    if let Some(b1_res) = get_char_s(&input, '[') {
        if let Some(b2_loc) = b1_res.find(']') {
            if let Some(el) = parse_jq(&b1_res[..b2_loc]) {
                if skip_stuff(el.1) == "" {
                    // Unpack because of the comma operator. It has a very high precedence.
                    return Some((Jq::Array(unpack_comma_to_array(el.0)), &b1_res[b2_loc+1..]))
                } else {
                    panic!("Bad Jq array")
                }
            } else {
                return Some((Jq::Iterator, &b1_res[b2_loc+1..]))
            }
        } else {
            panic!("No closing bracket ]")
        }
    } 
    None
}

fn parse_key_value(input: &str) -> Option<(String, Jq, &str)> {
    if let Some((Jq::String(key), k_res)) = parse_jq(input) {
        if let Some(d_res) = get_char_s(k_res, ':') {
            if let Some(value) = parse_jq(d_res) {
                return Some((key, value.0, value.1))
            }
        }
    }
    None
}

pub fn parse_object(input: &str) -> Option<(Jq, &str)> {
    if let Some(b1_res) = get_char_s(input, '{') {
        let mut map: HashMap<String, Jq> = HashMap::new();
        let mut parse_result = parse_key_value(b1_res);

        while let Some(el) = parse_result {
            map.insert(el.0, el.1);
            if let Some(c_res) = get_char_s(el.2, ',') {
                parse_result = parse_key_value(c_res);
            } else if let Some(b2_res) = get_char_s(el.2, '}') {
                return Some((Jq::Object(map), b2_res))
            } else {
                panic!("bad object")
            }
        }
    }
    None
}

fn parse_jq(input: &str) -> Option<(Jq, &str)> {
    if let Some(a) = parse_parenthesis(input) {
        return Some(a)
    }
    // Control
    if let Some(a) = parse_pipe(input) {
        return Some(a)
    }
    if let Some(a) = parse_comma(input) {
        return Some(a)
    }

    // Selection
    if let Some(a) = parse_optional(input) {
        return Some(a)
    }
    if let Some(a) = parse_recursive(input) {
        return Some(a)
    }
    if let Some(a) = parse_id_chain(input) {
        return Some(a)
    }
    if let Some(a) = parse_iterator(input) {
        return Some(a)
    }
    if let Some(a) = parse_slice(input) {
        return Some(a)
    }
    if let Some(a) = parse_identity(input) {
        return Some(a)
    }

    // Json
    if let Some(a) = parse_null(input) {
        return Some(a)
    }
    if let Some(a) = parse_boolean(input) {
        return Some(a)
    }
    if let Some(a) = parse_number(input) {
        return Some(a)
    }
    if let Some(a) = parse_string(input) {
        return Some(a)
    }
    if let Some(a) = parse_array(input) {
        return Some(a)
    }
    if let Some(a) = parse_object(input) {
        return Some(a)
    }

    None
}

pub fn parse(input: &str) -> Option<Jq> {
    if let Some((res, leftover)) = parse_jq(input) {
        if skip_stuff(leftover).len() == 0 {
            Some(res.clone())
        } else if leftover == input {
            None
        } else if let Some(rest) = parse(leftover) {
            Some(Jq::Pipe(Box::new(res.clone()), Box::new(rest)))
        } else {
            panic!("Failed to parse. Leftover: {}", leftover)
        }
    } else {
        None
    }
}