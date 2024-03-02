use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::json::json_print::print;

#[derive(Debug)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>)
}

impl Display for Json {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = write!(f, "");
        let res = print(self);
        for l1 in res {
            for l2 in l1 {
                writer = writer.and(write!(f, "{}", l2));
            }
            writer = writer.and(write!(f, "\n"));
        }
        writer
    }
}

impl Clone for Json {
    fn clone(&self) -> Self {
        match self {
            Json::Null => Json::Null,
            Json::Boolean(b) => Json::Boolean(*b),
            Json::Number(n) => Json::Number(*n),
            Json::String(s) => Json::String(s.clone()),
            Json::Array(arr) => Json::Array(arr.clone()),
            Json::Object(map) => Json::Object(map.clone())
        }
    }
}

impl PartialEq for Json {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialOrd for Json {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0.partial_cmp(r0),
            (Self::Number(l0), Self::Number(r0)) => l0.partial_cmp(r0),
            (Self::String(l0), Self::String(r0)) => l0.partial_cmp(r0),
            (Self::Array(l0), Self::Array(r0)) => l0.partial_cmp(r0),
            _ => panic!("Incomparable: {} {}", self, other)
        }
    }
}