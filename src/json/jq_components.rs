use std::{collections::HashMap, hash::{Hash, Hasher}};

use super::json_components::Json;

#[derive(Debug)]
pub enum Jq {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Jq>),
    Object(HashMap<Jq, Jq>),

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
    Slice(Option<usize>, Option<usize>),

    Input,

    Addition(Box<Jq>, Box<Jq>),
    Subtraction(Box<Jq>, Box<Jq>),
    Multiplication(Box<Jq>, Box<Jq>),
    Division(Box<Jq>, Box<Jq>),
    Modulo(Box<Jq>, Box<Jq>),

    Eq(Box<Jq>, Box<Jq>),
    NotEq(Box<Jq>, Box<Jq>),
    Gt(Box<Jq>, Box<Jq>),
    Gte(Box<Jq>, Box<Jq>),
    Lt(Box<Jq>, Box<Jq>),
    Lte(Box<Jq>, Box<Jq>),

    IfStatement(Box<Jq>, Box<Jq>, Option<Box<Jq>>),
    And(Box<Jq>, Box<Jq>),
    Or(Box<Jq>, Box<Jq>),
    Not(Box<Jq>),

    Alternative(Box<Jq>, Box<Jq>),

    Abs(Option<Box<Jq>>),
    Length(Option<Box<Jq>>),
    Keys(Option<Box<Jq>>),
    Has(Box<Jq>),
    In(Box<Jq>),
    Map(Box<Jq>),
}

impl From<Json> for Jq {
    fn from(value: Json) -> Self {
        match value {
            Json::Null => Jq::Null,
            Json::Boolean(b) => Jq::Boolean(b),
            Json::Number(n) => Jq::Number(n),
            Json::String(s) => Jq::String(s),
            Json::Array(a) => Jq::Array(a.into_iter().map(Self::from).collect()),
            Json::Object(o) => Jq::Object(o.into_iter().map(|(key, val)| (Jq::String(key), Self::from(val))).collect())
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
            Jq::Object(o) => Json::Object(o.into_iter().map(|(key, val)|  match key {
                Jq::String(str) => (str, Self::from(val)),
                _ => panic!("Can only use strings as object keys")
            }).collect()),
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
            Jq::IsNull => Jq::IsNull,
            Jq::Addition(a, b) => Jq::Addition(a.clone(), b.clone()),
            Jq::Subtraction(a, b) => Jq::Subtraction(a.clone(), b.clone()),
            Jq::Multiplication(a, b) => Jq::Multiplication(a.clone(), b.clone()),
            Jq::Division(a, b) => Jq::Division(a.clone(), b.clone()),
            Jq::Modulo(a, b) => Jq::Modulo(a.clone(), b.clone()),
            Jq::Eq(a, b) => Jq::Eq(a.clone(), b.clone()),
            Jq::NotEq(a, b) => Jq::NotEq(a.clone(), b.clone()),
            Jq::Gt(a, b) => Jq::Gt(a.clone(), b.clone()),
            Jq::Gte(a, b) => Jq::Gte(a.clone(), b.clone()),
            Jq::Lt(a, b) => Jq::Lt(a.clone(), b.clone()),
            Jq::Lte(a, b) => Jq::Lte(a.clone(), b.clone()),
            Jq::IfStatement(a, b, c) => Jq::IfStatement(a.clone(), b.clone(), c.clone()),
            Jq::And(a, b) => Jq::And(a.clone(), b.clone()),
            Jq::Or(a, b) => Jq::Or(a.clone(), b.clone()),
            Jq::Not(a) => Jq::Not(a.clone()),
            Jq::Alternative(a, b) => Jq::Alternative(a.clone(), b.clone()),
            Jq::Abs(a) => Jq::Abs(a.clone()),
            Jq::Length(a) => Jq::Length(a.clone()),
            Jq::Keys(a) => Jq::Keys(a.clone()),
            Jq::Has(a) => Jq::Has(a.clone()),
            Jq::In(a) => Jq::In(a.clone()),
            Jq::Map(a) => Jq::Map(a.clone()),
        }
    }
}

impl PartialEq for Jq {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0.to_bits() == r0.to_bits(),
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            (Self::Pipe(l0, l1), Self::Pipe(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Comma(l0, l1), Self::Comma(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Id(l0), Self::Id(r0)) => l0 == r0,
            (Self::IdChain(l0), Self::IdChain(r0)) => l0 == r0,
            (Self::Optional(l0), Self::Optional(r0)) => l0 == r0,
            (Self::Parenthesis(l0), Self::Parenthesis(r0)) => l0 == r0,
            (Self::Slice(l0, l1), Self::Slice(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Addition(l0, l1), Self::Addition(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Subtraction(l0, l1), Self::Subtraction(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Multiplication(l0, l1), Self::Multiplication(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Division(l0, l1), Self::Division(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Modulo(l0, l1), Self::Modulo(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Eq(l0, l1), Self::Eq(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::NotEq(l0, l1), Self::NotEq(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Gt(l0, l1), Self::Gt(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Gte(l0, l1), Self::Gte(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Lt(l0, l1), Self::Lt(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Lte(l0, l1), Self::Lte(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::IfStatement(l0, l1, l2), Self::IfStatement(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
            (Self::And(l0, l1), Self::And(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Or(l0, l1), Self::Or(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Not(l0), Self::Not(r0)) => l0 == r0,
            (Self::Alternative(l0, l1), Self::Alternative(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Abs(l0), Self::Abs(r0)) => l0 == r0,
            (Self::Length(l0), Self::Length(r0)) => l0 == r0,
            (Self::Keys(l0), Self::Keys(r0)) => l0 == r0,
            (Self::Has(l0), Self::Has(r0)) => l0 == r0,
            (Self::In(l0), Self::In(r0)) => l0 == r0,
            (Self::Map(l0), Self::Map(r0)) => l0 == r0,

            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for Jq {}

impl Hash for Jq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Jq::Null => 0.hash(state),
            Jq::Boolean(b) => {
                1.hash(state);
                b.hash(state);
            }
            Jq::Number(n) => {
                2.hash(state);
                // Hashing f64 as u64 to avoid issues with floating point precision
                let mut buf = [0; 8];
                let bytes = n.to_bits().to_ne_bytes();
                buf.copy_from_slice(&bytes);
                buf.hash(state);
            }
            Jq::String(s) => {
                3.hash(state);
                s.hash(state);
            }
            Jq::Array(arr) => {
                4.hash(state);
                for jq in arr {
                    jq.hash(state);
                }
            }
            Jq::Object(obj) => {
                5.hash(state);
                // Objects can contain Jq types, which needs to be recursively hashed
                for (key, value) in obj {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Jq::Pipe(left, right)
            | Jq::Comma(left, right) => {
                6.hash(state);
                left.hash(state);
                right.hash(state);
            }
            Jq::Id(id) => {
                7.hash(state);
                id.hash(state);
            }
            Jq::IdChain(chain) => {
                8.hash(state);
                for jq in chain {
                    jq.hash(state);
                }
            }
            Jq::Identity => 9.hash(state),
            Jq::IsNull => 10.hash(state),
            Jq::Iterator => 11.hash(state),
            Jq::Recursive => 12.hash(state),
            Jq::Input => 13.hash(state),
            Jq::Slice(start, end) => {
                14.hash(state);
                start.hash(state);
                end.hash(state);
            }

            Jq::Optional(left)
            | Jq::Parenthesis(left) => {
                15.hash(state);
                left.hash(state)
            }
            Jq::Addition(a, b) => {
                16.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Subtraction(a, b) => {
                17.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Multiplication(a, b) => {
                18.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Division(a, b) => {
                19.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Modulo(a, b) => {
                20.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Eq(a, b) => {
                21.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::NotEq(a, b) => {
                22.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Gt(a, b) => {
                23.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Gte(a, b) => {
                24.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Lt(a, b) => {
                25.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Lte(a, b) => {
                26.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::IfStatement(a, b, c) => {
                27.hash(state);
                a.hash(state);
                b.hash(state);
                c.hash(state);
            },
            Jq::And(a, b) => {
                28.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Or(a, b) => {
                29.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Not(a) => {
                30.hash(state);
                a.hash(state);
            },
            Jq::Alternative(a, b) => {
                31.hash(state);
                a.hash(state);
                b.hash(state);
            },
            Jq::Length(a) => {
                31.hash(state);
                a.hash(state);
            },
            Jq::Abs(a) => {
                32.hash(state);
                a.hash(state);
            },
            Jq::Keys(a) => {
                33.hash(state);
                a.hash(state);
            },
            Jq::Has(a) => {
                34.hash(state);
                a.hash(state);
            },
            Jq::In(a) => {
                35.hash(state);
                a.hash(state);
            },
            Jq::Map(a) => {
                36.hash(state);
                a.hash(state);
            },
        }
    }
    
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.hash(state)
        }
    }
}