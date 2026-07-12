use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigValueType {
    String,
    Bool,
    Float,
    Int,
}
impl Display for ConfigValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::String => "string",
            Self::Bool => "bool",
            Self::Float => "float",
            Self::Int => "int",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue<'c> {
    String(&'c str),
    Bool(bool),
    Float(f64),
    Int(i64),
}
impl<'c> ConfigValue<'c> {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn ty(&self) -> ConfigValueType {
        match self {
            Self::String(_) => ConfigValueType::String,
            Self::Bool(_) => ConfigValueType::Bool,
            Self::Float(_) => ConfigValueType::Float,
            Self::Int(_) => ConfigValueType::Int,
        }
    }
}
