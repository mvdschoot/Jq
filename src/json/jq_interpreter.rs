use std::collections::HashMap;

use crate::json::jq_components::Jq;
use crate::json::json_components::Json;

type InterpretResult = Result<Vec<Json>, String>;

macro_rules! err {
    ($fmt:expr $(, $arg:expr)* ) => {
        Err(format!($fmt $(, $arg)*))
    };
}

fn interpret_pipe(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret(interpret(vec![json], *a)?, *b)
}

fn interpret_comma(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let mut res: Vec<Json> = interpret(vec![json.clone()], *a)?;
    res.extend(interpret(vec![json], *b)?);
    return Ok(res)
}

fn interpret_id(json: Json, id: String) -> InterpretResult {
    match &json {
        Json::Object(map) => if let Some(value) = map.get(&id) {
            Ok(vec![value.clone()])
        } else {
            err!("Can only index an object with an id")
        },
        _ => err!("Bad indexing with ID. The json is {}", json)
    }
}

fn interpret_id_chain(json: Json, chain: &[Jq]) -> InterpretResult {
    if let Some(id) = chain.first() {
        match &id {
            Jq::Id(id) => match &json {
                Json::Object(map) => if let Some(value) = map.get(id) {
                        interpret_id_chain(value.clone(), &chain[1..])
                    } else {
                        return Ok(vec![Json::Null])
                    }
                _ => err!("Is not an object. The ID to retrieve is '{}'. The json is '{}'", id, json)
            }
            Jq::Array(arr) => {
                let mut res: Vec<Json> = Vec::new(); 
                for arr_id in arr {
                    match (arr_id, &json) {
                        (Jq::String(id), Json::Object(map)) => if let Some(value) = map.get(id) {
                            res.extend(interpret_id_chain(value.clone(), &chain[1..])?);
                        },
                        (Jq::Number(num), Json::Array(arr)) => {
                            if let Some(value) = arr.iter().nth(num.floor() as usize) {
                                res.extend(interpret_id_chain(value.clone(), &chain[1..])?)
                            } else {
                                return Ok(vec![Json::Null])
                            }
                        },
                        (Jq::Slice(a, b), any_json) => {
                            if let Ok(slice) = interpret_slice(any_json.clone(), *a, *b) {
                                res.extend(interpret_id_chain(Json::Array(slice.to_vec()), &chain[1..])?)
                            }
                        }
                        _ => return err!("Bad indexing")
                    }
                }
                Ok(res)
            },
            Jq::Iterator => {
                let it_res = interpret_iterator(json)?;
                let mut res: Vec<Json> = Vec::new();
                for el in it_res {
                    res.extend(interpret_id_chain(el, &chain[1..])?)
                }
                Ok(res)
            }
            _ => err!("Cannot interpret this id in the chain: {:?}", id)
        }
    } else {
        Ok(vec![json])
    }
}

fn interpret_identity(json: Json) -> Json {
    json
}


fn interpret_optional(json: Json, input: Box<Jq>) -> InterpretResult {
    match interpret(vec![json], *input) {
        Ok(res) => Ok(res),
        Err(_) => Ok(vec![Json::Null])
    }
}

fn interpret_recursive(json: Json) -> Vec<Json> {
    match &json {
        Json::Array(arr) => {
            let mut res: Vec<Json> = vec![json.clone()];
            arr.iter().for_each(|el| {
                res.extend(interpret_recursive(el.clone()));
            });
            res
        },
        Json::Object(map) => {
            let mut res: Vec<Json> = vec![json.clone()];
            map.iter().for_each(|el| {
                res.extend(interpret_recursive(el.1.clone()));
            });
            res
        },
        _ => vec![json]
    }
}

fn interpret_iterator(json: Json) -> InterpretResult {
    match json {
        Json::Array(arr) => interpret(arr, Jq::Identity),
        Json::Object(map) => interpret(map.values().cloned().collect(), Jq::Identity),
        _ => err!("Cannot iterate over {}", json)
    }
}

fn interpret_slice(json: Json, a: Option<usize>, b: Option<usize>) -> InterpretResult {
    match json {
        Json::Array(arr) => {
            // a and b should be type usize, so never < 0.
            match (a, b) {
                (Some(va), Some(vb)) if va < vb && vb < arr.len() => Ok(arr[va..vb].to_vec()),
                (Some(va), None) if va < arr.len() => Ok(arr[va..].to_vec()),
                (None, Some(vb)) if vb < arr.len() => Ok(arr[..vb].to_vec()),
                _ => panic!("Invalid slicing")
            }
        }
        _ => err!("Can only use slice on a JSON array.")
    }
}

fn interpret_json_array(json: Json, jq: Vec<Jq>) -> InterpretResult {
    let mut res: Vec<Json> = Vec::new();
    for j in jq {
        match interpret(vec![json.clone()], j) {
            Ok(r) => res.extend(r),
            Err(err) => return Err(err)
        }
    }
    Ok(vec![Json::Array(res)])
}

fn interpret_json_object(json: Json, jq: HashMap<Jq, Jq>) -> InterpretResult {
    let mut res: Vec<Json> = Vec::new();
    for (key, value) in jq {
        match (interpret(vec![json.clone()], key), interpret(vec![json.clone()], value)) {
            (Ok(a), Ok(b)) => {
                for a_res in a {
                    if let Json::String(key_string) = &a_res {
                        for b_res in b.clone() {
                            let mut hash_res: HashMap<String, Json> = HashMap::new();
                            hash_res.insert(key_string.clone(), b_res);
                            res.push(Json::Object(hash_res));
                        }
                    } else {
                        panic!("Object key has to evaluate to a string.")
                    }
                }
            },
            (Err(err), _) => return Err(err),
            (_, Err(err)) => return Err(err)
        }
    }
    Ok(res)
}

fn interpret_addition(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            match (a_el.clone(), b_el.clone()) {
                (Json::Number(a_num), Json::Number(b_num)) => res.push(Json::Number(a_num + b_num)),
                (Json::Array(mut a_arr), Json::Array(b_arr)) => res.push(Json::Array({a_arr.extend(b_arr); a_arr})),
                (Json::String(a_str), Json::String(b_str)) => res.push(Json::String(a_str.clone() + &b_str)),
                (Json::Object(mut a_obj), Json::Object(b_obj)) => res.push(Json::Object({a_obj.extend(b_obj.clone()); a_obj})),
                (a_any, Json::Null) => res.push(a_any),
                _ => panic!("Bad addition")
            }
        }
    }
    Ok(res)
}

fn interpret_subtraction(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            match (a_el.clone(), b_el.clone()) {
                (Json::Number(a_num), Json::Number(b_num)) => res.push(Json::Number(a_num - b_num)),
                (Json::Array(mut a_arr), Json::Array(b_arr)) => res.push(Json::Array({a_arr.retain(|el| !b_arr.contains(el)); a_arr})),
                _ => panic!("Bad subtraction")
            }
        }
    }
    Ok(res)
}

fn interpret_json_multiplication(a: Json, b: Json) -> Result<Json, String> {
    match (&a, &b) {
        (Json::Number(a_num), Json::Number(b_num)) => Ok(Json::Number(a_num * b_num)),

        (Json::Number(a_num), Json::String(b_str)) => Ok(Json::String(b_str.repeat(a_num.floor() as usize))),
        (Json::String(a_str), Json::Number(b_num)) => Ok(Json::String(a_str.repeat(b_num.floor() as usize))),

        (Json::Object(a_obj), Json::Object(b_obj)) => {
            let mut map = a_obj.clone();
            for b_el in b_obj {
                if map.contains_key(b_el.0) {
                    map.insert(b_el.0.clone(), interpret_json_multiplication(a_obj.get(b_el.0).unwrap().clone(), b_el.1.clone())?);
                } else {
                    map.insert(b_el.0.clone(), b_el.1.clone());
                }
            }
            Ok(Json::Object(map))
        },
        _ => err!("Bad multiplication")
    }
}

fn interpret_multiplication(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json.clone()], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            res.push(interpret_json_multiplication(a_el.clone(), b_el.clone())?);
        }
    }
    Ok(res)
}

fn interpret_division(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            match (a_el.clone(), b_el.clone()) {
                (Json::Number(a_num), Json::Number(b_num)) => res.push(Json::Number(a_num / b_num)),
                _ => panic!("Bad division")
            }
        }
    }
    Ok(res)
}

fn interpret_modulo(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            match (a_el.clone(), b_el.clone()) {
                (Json::Number(a_num), Json::Number(b_num)) => res.push(Json::Number(a_num % b_num)),
                _ => panic!("Bad subtraction")
            }
        }
    }
    Ok(res)
}

fn interpret_boolean_op(json: Json, a: Box<Jq>, b: Box<Jq>, comp: fn(&Json, &Json) -> bool) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;
    let b = interpret(vec![json], *b)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        for b_el in &b {
            res.push(Json::Boolean(comp(a_el, b_el)))
        }
    }
    Ok(res)
}

fn interpret_eq(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f == s)
}

fn interpret_not_eq(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f != s)
}

fn interpret_gt(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f > s)
}

fn interpret_gte(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f >= s)
}

fn interpret_lt(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f < s)
}

fn interpret_lte(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| f <= s)
}

fn interpret_if_statement(json: Json, i: Box<Jq>, t: Box<Jq>, e: Option<Box<Jq>>) -> InterpretResult {
    let if_res = interpret(vec![json.clone()], *i)?;

    let mut res: Vec<Json> = Vec::new();
    for if_r  in if_res {
        match if_r {
            Json::Null | Json::Boolean(false) => {
                if let Some(else_statement) = &e {
                    res.extend(interpret(vec![json.clone()], *else_statement.clone())?);
                } else {
                    res.push(json.clone());
                }
            },
            _anything_else => res.extend(interpret(vec![json.clone()], *t.clone())?),
        }
    }
    Ok(res)
}

fn interpret_and(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| {
        match (f, s) {
            (Json::Boolean(a_res), Json::Boolean(b_res)) => *a_res && *b_res,
            _ => false
        }
    })
}

fn interpret_or(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    interpret_boolean_op(json, a, b, |f, s| {
        match (f, s) {
            (Json::Boolean(a_res), Json::Boolean(b_res)) => *a_res || *b_res,
            _ => false
        }
    })
}

fn interpret_not(json: Json, a: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json], *a)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in a {
        match a_el {
            Json::Boolean(boo) => res.push(Json::Boolean(!boo)),
            _ => return err!("Can't use 'not' on '{}'", a_el)
        };
    }
    Ok(res)
}

fn interpret_alternative(json: Json, a: Box<Jq>, b: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a);
    if a.is_err() {
        return interpret(vec![json], *b)
    }

    let a = a.unwrap();

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        match a_el {
            Json::Boolean(false) | Json::Null => res.extend(interpret(vec![json.clone()], *b.clone())?),
            anything_else => res.push(anything_else.clone())
        }
    }
    Ok(res)
}

macro_rules! map_or_interpret {
    ($a:expr, $json:expr) => {
        $a.map_or_else(|| Ok(vec![$json.clone()]), |el| interpret(vec![$json.clone()], *el))?
    };
}


fn interpret_abs(json: Json, a: Option<Box<Jq>>) -> InterpretResult {
    let a = map_or_interpret!(a, json);

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        match a_el {
            Json::Number(num) => res.push(Json::Number(num.abs())),
            _ => return err!("Cannot take absolute value of '{}'", a_el)
        }
    }
    Ok(res)
}

fn interpret_length(json: Json, a: Option<Box<Jq>>) -> InterpretResult {
    let a = map_or_interpret!(a, json);

    let mut res: Vec<Json> = Vec::new();
    for a_el in a {
        match a_el {
            Json::Number(num) => res.push(Json::Number(num.abs())),
            Json::String(str) => res.push(Json::Number(str.as_bytes().len() as f64)),
            Json::Array(arr) => res.push(Json::Number(arr.len() as f64)),
            Json::Object(obj) => res.push(Json::Number(obj.len() as f64)),
            Json::Null => res.push(Json::Number(0.0)),
            _ => return err!("Cannot take absolute value of '{}'", a_el)
        }
    }
    Ok(res)
}

fn interpret_keys(json: Json, a: Option<Box<Jq>>) -> InterpretResult {
    let a = map_or_interpret!(a, json);

    let mut res: Vec<Json> = Vec::new();
    for a_el in a {
        match a_el {
            Json::Object(obj) => res.push(Json::Array(obj.keys().into_iter().map(|key| Json::String(key.clone())).collect())),
            _ => return err!("Cannot get keys of '{}'", a_el)
        }
    }
    Ok(res)
}

fn interpret_has(json: Json, a: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        match (&json, a_el) {
            (Json::Object(obj), Json::String(key)) => res.push(Json::Boolean(obj.contains_key(key))),
            (Json::Array(arr), Json::Number(index)) => res.push(Json::Boolean(arr.get(index.floor() as usize).is_some())),
            _ => return err!("Invalid use of 'has' function input: '{}', argument: '{}'", json, a_el)
        }
    }
    Ok(res)
}

fn interpret_in(json: Json, a: Box<Jq>) -> InterpretResult {
    let a = interpret(vec![json.clone()], *a)?;

    let mut res: Vec<Json> = Vec::new();
    for a_el in &a {
        match (&a_el, &json) {
            (Json::Object(obj), Json::String(key)) => res.push(Json::Boolean(obj.contains_key(key))),
            (Json::Array(arr), Json::Number(index)) => res.push(Json::Boolean(arr.get(index.floor() as usize).is_some())),
            _ => return err!("Invalid use of 'in' function input: '{}', argument: '{}'", json, a_el)
        }
    }
    Ok(res)
}

fn interpret_map(json: Json, a: Box<Jq>) -> InterpretResult {
    let mut res: Vec<Json> = Vec::new();
    match &json {
        Json::Object(obj) => res.extend(interpret(obj.values().cloned().collect(), *a)?),
        Json::Array(arr) => res.extend(interpret(arr.clone(), *a)?),
        other => return err!("Invalid use of 'map' function input: '{}', argument: '{:?}'", other, *a)
    }
    Ok(res)
}

pub fn interpret(json: Vec<Json>, jq: Jq) -> InterpretResult {
    // println!("{:?}\t\t\t {:?}", json, jq);
    let mut res: Vec<Json> = Vec::new();
    for json_el in json {
        match &jq {
            Jq::Pipe(a, b) => res.extend(interpret_pipe(json_el, a.clone(), b.clone())?),
            Jq::Comma(a, b) => res.extend(interpret_comma(json_el, a.clone(), b.clone())?),
            Jq::Id(a) => res.extend(interpret_id(json_el, a.clone())?),
            Jq::IdChain(a) => res.extend(interpret_id_chain(json_el, a)?),
            Jq::Identity => res.push(interpret_identity(json_el)),
            Jq::Optional(a) => res.extend(interpret_optional(json_el, a.clone())?),
            Jq::Iterator => res.extend(interpret_iterator(json_el)?),
            Jq::Recursive => res.extend(interpret_recursive(json_el)),
            Jq::Parenthesis(a) => res.extend(interpret(vec![json_el], *a.clone())?),
            Jq::Slice(a, b) => res.extend(interpret_slice(json_el, a.clone(), b.clone())?),

            Jq::Input => res.push(json_el),
            Jq::IsNull => unreachable!("vgm mag ie hier niet komen"),

            Jq::Array(arr) => res.extend(interpret_json_array(json_el, arr.clone())?),
            Jq::Object(map) => res.extend(interpret_json_object(json_el, map.clone())?),
            Jq::Null => res.push(Json::Null),
            Jq::Boolean(a) => res.push(Json::Boolean(*a)),
            Jq::Number(a) => res.push(Json::Number(*a)),
            Jq::String(a) => res.push(Json::String(a.clone())),

            Jq::Addition(a, b) => res.extend(interpret_addition(json_el, a.clone(), b.clone())?),
            Jq::Subtraction(a, b) => res.extend(interpret_subtraction(json_el, a.clone(), b.clone())?),
            Jq::Multiplication(a, b) => res.extend(interpret_multiplication(json_el, a.clone(), b.clone())?),
            Jq::Division(a, b) => res.extend(interpret_division(json_el, a.clone(), b.clone())?),
            Jq::Modulo(a, b) => res.extend(interpret_modulo(json_el, a.clone(), b.clone())?),

            Jq::Eq(a, b) => res.extend(interpret_eq(json_el, a.clone(), b.clone())?),
            Jq::NotEq(a, b) => res.extend(interpret_not_eq(json_el, a.clone(), b.clone())?),
            Jq::Gt(a, b) => res.extend(interpret_gt(json_el, a.clone(), b.clone())?),
            Jq::Gte(a, b) => res.extend(interpret_gte(json_el, a.clone(), b.clone())?),
            Jq::Lt(a, b) => res.extend(interpret_lt(json_el, a.clone(), b.clone())?),
            Jq::Lte(a, b) => res.extend(interpret_lte(json_el, a.clone(), b.clone())?),

            Jq::IfStatement(a, b, c) => res.extend(interpret_if_statement(json_el, a.clone(), b.clone(), c.clone())?),
            Jq::And(a, b) => res.extend(interpret_and(json_el, a.clone(), b.clone())?),
            Jq::Or(a, b) => res.extend(interpret_or(json_el, a.clone(), b.clone())?),
            Jq::Not(a) => res.extend(interpret_not(json_el, a.clone())?),

            Jq::Alternative(a, b) => res.extend(interpret_alternative(json_el, a.clone(), b.clone())?),

            Jq::Length(a) => res.extend(interpret_length(json_el, a.clone())?),
            Jq::Abs(a) => res.extend(interpret_abs(json_el, a.clone())?),
            Jq::Keys(a) => res.extend(interpret_keys(json_el, a.clone())?),
            Jq::Has(a) => res.extend(interpret_has(json_el, a.clone())?),
            Jq::In(a) => res.extend(interpret_in(json_el, a.clone())?),
            Jq::Map(a) => res.extend(interpret_map(json_el, a.clone())?),
        };
    }
    Ok(res)
}