use std::fmt::Display;

use felt::Felt252;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Type {
    Felt252(Felt252),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    Bool(bool),

    Vector(Box<Type>, Vec<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Felt252(_)
            | Type::U8(_)
            | Type::U16(_)
            | Type::U32(_)
            | Type::U64(_)
            | Type::U128(_)
            | Type::Bool(_) => write!(f, "{:?}", self),
            Type::Vector(t, v) => match **t {
                Type::Felt252(_) => {
                    let buffer: Vec<Felt252> = v
                        .iter()
                        .map(|v| {
                            if let Type::Felt252(a) = v {
                                a.to_owned()
                            } else {
                                todo!()
                            }
                        })
                        .collect();
                    if buffer.len() > 0 {
                        write!(f, "Vector(Felt252, [{:?}])", buffer)
                    } else {
                        write!(f, "Vector(Felt252)")
                    }
                }
                _ => todo!(),
            },
        }
    }
}

pub struct Parameters(pub Vec<Type>);

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ").unwrap();
        for v in self.0.clone() {
            write!(f, "{}", v).unwrap();
            if v != *self.0.last().unwrap() {
                write!(f, ", ").unwrap();
            }
        }
        write!(f, " ]")
    }
}
