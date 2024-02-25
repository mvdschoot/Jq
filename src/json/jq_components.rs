use std::collections::HashMap;

use super::json_components::Json;

#[derive(Debug)]
pub enum Jq {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Jq>),
    Object(HashMap<String, Jq>),

    Pipe(Box<Jq>, Box<Jq>),
    Comma(Box<Jq>, Box<Jq>),
    Id(String),
    IdChain(Vec<Jq>), // Either id or array
    Identity,
    Optional(Box<Jq>),
    IsNull,
    Iterator,
    Recursive,
    Parenthesis(Box<Jq>),
    Slice(usize, usize),

    Input,
}

impl From<Json> for Jq {
    fn from(value: Json) -> Self {
        match value {
            Json::Null => Jq::Null,
            Json::Boolean(b) => Jq::Boolean(b),
            Json::Number(n) => Jq::Number(n),
            Json::String(s) => Jq::String(s),
            Json::Array(a) => Jq::Array(a.into_iter().map(Self::from).collect()),
            Json::Object(o) => Jq::Object(o.into_iter().map(|(key, val)| (key, Self::from(val))).collect())
        }
    }
}

impl From<Jq> for Json {
    fn from(value: Jq) -> Self {
        match value {
            Jq::Null => Json::Null,
            Jq::Boolean(b) => Json::Boolean(b),
            Jq::Number(n) => Json::Number(n),
            Jq::String(s) => Json::String(s),
            Jq::Array(a) => Json::Array(a.into_iter().map(Self::from).collect()),
            Jq::Object(o) => Json::Object(o.into_iter().map(|(key, val)| (key, Self::from(val))).collect()),
            _ => panic!("Cannot convert {:?} to json", value)
        }
    }
}

impl Clone for Jq {
    fn clone(&self) -> Self {
        match self {
            Jq::Null => Jq::Null,
            Jq::Boolean(b) => Jq::Boolean(b.clone()),
            Jq::Number(n) => Jq::Number(n.clone()),
            Jq::String(s) => Jq::String(s.clone()),
            Jq::Array(arr) => Jq::Array(arr.clone()),
            Jq::Object(map) => Jq::Object(map.clone()),
            Jq::Pipe(a, b) => Jq::Pipe(a.clone(), b.clone()),
            Jq::Comma(a, b) => Jq::Comma(a.clone(), b.clone()),
            Jq::Id(a) => Jq::Id(a.clone()),
            Jq::IdChain(a) => Jq::IdChain(a.clone()),
            Jq::Identity => Jq::Identity,
            Jq::Optional(a) => Jq::Optional(a.clone()),
            Jq::Iterator => Jq::Iterator,
            Jq::Recursive => Jq::Recursive,
            Jq::Parenthesis(a) => Jq::Parenthesis(a.clone()),
            Jq::Slice(a, b) => Jq::Slice(a.clone(), b.clone()),
            Jq::Input => Jq::Input,
            Jq::IsNull => Jq::IsNull
        }
    }
}