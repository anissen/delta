use std::fmt::Display;

use elements::{ComponentLayout, FieldLayout};

// TODO(anissen): See https://github.com/brightly-salty/rox/blob/master/src/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    True,
    False,
    Integer(i32),
    Float(f32),
    String(String),
    SimpleTag(String),
    Tag(String, Box<Value>),
    List(Vec<Value>),
    Function(u8),
    Component { id: u8, properties: Vec<Value> },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::True => write!(f, "true")?,
            Value::False => write!(f, "false")?,
            Value::Integer(i) => write!(f, "{i}")?,
            Value::Float(d) => write!(f, "{d:.2}")?,
            Value::String(s) => write!(f, "{s}")?,
            Value::SimpleTag(t) => write!(f, "{t}")?,
            Value::Tag(t, a) => write!(f, "{t}({a})")?,
            Value::List(l) => {
                let mut first = true;
                write!(f, "[")?;
                for v in l {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                    first = false;
                }
                write!(f, "]")?;
            }
            Value::Function(i) => write!(f, "<fn {i}>")?,
            Value::Component { id, properties } => {
                let properties_str = properties
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "component({id}, {properties_str})")?;
            }
        };
        Ok(())
    }
}

impl Value {
    // get type id
    pub fn type_id(&self) -> u8 {
        match self {
            Value::True | Value::False => 0,
            Value::Integer(_) => 1,
            Value::Float(_) => 2,
            Value::String(_) => 3,
            Value::SimpleTag(_) => 4,
            Value::Tag(_, _) => 5,
            Value::List(_) => 6,
            Value::Function(_) => 7,
            Value::Component { .. } => 8,
        }
    }
}

fn read_f32(b: &[u8]) -> f32 {
    f32::from_be_bytes(b.try_into().unwrap())
}

fn read_i32(b: &[u8]) -> i32 {
    i32::from_be_bytes(b.try_into().unwrap())
}

fn read_byte(b: &[u8]) -> u8 {
    b[0]
}

fn read_string(b: &[u8]) -> String {
    let length = b[0] as usize;
    let bytes: Vec<u8> = b[1..length + 1].into();

    String::from_utf8(bytes).unwrap()
}

pub fn get_values_from_bytes(data: &[u8], layout: &ComponentLayout) -> Vec<Value> {
    let mut offset = 0;
    layout
        .fields
        .iter()
        .map(|field| {
            let size = field.size as usize;
            let bytes = &data[offset..offset + size];
            offset += size;
            get_value_from_bytes(bytes, field)
        })
        .collect()
}

pub fn get_value_from_bytes(data: &[u8], field: &FieldLayout) -> Value {
    match field.type_id {
        0 => {
            if read_byte(data) != 0 {
                Value::True
            } else {
                Value::False
            }
        }
        1 => Value::Integer(read_i32(data)),
        2 => Value::Float(read_f32(data)),
        3 => Value::String(read_string(data)),
        _ => panic!("unknown type id"),
    }
}

pub fn get_bytes_from_values(values: &Vec<Value>, layout: &ComponentLayout) -> Vec<u8> {
    let mut bytes = Vec::new();
    layout.fields.iter().enumerate().for_each(|(index, field)| {
        let value = &values[index];
        let value_bytes = get_bytes_from_value(value, field);
        bytes.extend_from_slice(&value_bytes);
    });
    bytes
}

pub fn get_bytes_from_value(value: &Value, field_layout: &FieldLayout) -> Vec<u8> {
    let mut bytes = Vec::new();
    match field_layout.type_id {
        0 => match value {
            Value::True => bytes.push(1),
            Value::False => bytes.push(0),
            _ => panic!("Expected boolean property"),
        },
        1 => match value {
            Value::Integer(value) => bytes.extend_from_slice(&value.to_be_bytes()),
            _ => panic!("Expected integer property"),
        },
        2 => match value {
            Value::Float(value) => bytes.extend_from_slice(&value.to_be_bytes()),
            _ => panic!("Expected float property"),
        },
        3 => match value {
            Value::String(value) => {
                if value.len() > 32 {
                    panic!("String too long");
                }
                bytes.push(value.len() as u8);
                bytes.extend_from_slice(value.as_bytes()); // TODO(anissen): What about byte order?
                for _ in bytes.len()..33 {
                    bytes.push(0);
                }
            }
            _ => panic!("Expected string property"),
        },
        _ => panic!("Unsupported type"),
    };
    bytes.to_vec()
}
