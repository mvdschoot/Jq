use std::collections::HashMap;
use std::vec;

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
                        err!("Key {} does not exist in object {:?}", id, map)
                    },
                Json::Array(arr) => {
                    let mut result: Vec<Json> = Vec::new();
                    for el in arr {
                        if let Ok(el_res) = interpret_id_chain(el.clone(), chain) {
                            result.extend(el_res)
                        } else {
                            return err!("Problem in ID chain");
                        }
                    }
                    Ok(result)
                }
                _ => err!("Is not an object. id to get is {}. Json is {}", id, json)
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
                                return err!("Index {:?} does not exist in array {:?}", id, arr)
                            }
                        },
                        (Jq::Slice(a, b), Json::Array(arr)) => {
                            if let Some(slice) = arr.get(*a..*b) {
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

fn interpret_slice(json: Json, a: usize, b: usize) -> InterpretResult {
    match &json {
        Json::Array(arr) => {
            let slice = &arr[a..b];
            Ok(Vec::from(slice))
        }
        o => err!("Cannot slice {}", o)
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

fn interpret_json_object(json: Json, jq: HashMap<String, Jq>) -> InterpretResult {
    let mut res: HashMap<String, Json> = HashMap::new();
    for j in jq {
        match interpret(vec![json.clone()], j.1) {
            Ok(r) => r.iter().for_each(|el| {res.insert(j.0.clone(), el.clone());}),
            Err(err) => return Err(err)
        }
    }
    Ok(vec![Json::Object(res)])
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

            Jq::Null => {}
            Jq::Boolean(_) => {}
            Jq::Number(_) => {}
            Jq::String(_) => {}
        };
    }
    Ok(res)
}