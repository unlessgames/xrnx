use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LuaKind {
    Nil,
    Unknown,
    Any,
    Boolean,
    String,
    Number,
    Integer,
    Function,
    Table,
    Thread,
    UserData,
    Binary,
    LightUserData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub name: String,
    pub kind: Kind,
    // values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Unresolved(String),
    Lua(LuaKind),
    Array(Box<Kind>),
    Nullable(Box<Kind>),
    Table(Box<Kind>, Box<Kind>),
    Object(HashMap<String, Box<Kind>>),
    Alias(String),
    Class(String),
    Function(Function),
    Enum(Vec<Kind>),
    EnumRef(String),
    SelfArg,
    Variadic(Box<Kind>),
    Literal(Box<LuaKind>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub name: Option<String>,
    pub kind: Kind,
    pub desc: Option<String>,
    // pub default: String,
    // pub range: String
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<Var>,
    pub returns: Vec<Var>,
    pub desc: Option<String>,
    // pub overloads: ?
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: String,
    pub fields: Vec<Var>,
    pub methods: Vec<Function>,
    pub enums: Vec<Enum>,
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Def {
    Class(Class),
    Enum(Enum),
    Alias(Alias),
    Function(Function),
}

impl LuaKind {
    pub fn show(&self) -> String {
        let s = serde_json::to_string(self).unwrap();
        s.trim_matches('"').to_string()
    }
}

impl Var {
    pub fn show(&self) -> String {
        format!(
            "Var {} : {}",
            self.name.clone().unwrap_or_default(),
            self.kind
        )
    }
}

impl Enum {
    pub fn show(&self) -> String {
        format!("Enum {}", self.name)
    }
}

// impl Alias {
//     pub fn show(&self) -> String {
//         format!("Alias {} {}", self.name, self.kind)
//     }
// }

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(_) => write!(f, "\x1b[33m{:?}\x1b[0m", self),
            Self::Nullable(b) => write!(f, "Nullable({})", b.as_ref()),
            Self::Array(b) => write!(f, "Array({})", b.as_ref()),
            Self::Table(k, v) => write!(f, "Table({}, {})", k.as_ref(), v.as_ref()),
            Self::Enum(ks) => write!(
                f,
                "Enum({})",
                ks.iter()
                    .map(|k| format!("{}", k))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(hm) => {
                write!(
                    f,
                    "Object({})",
                    hm.iter()
                        .map(|(k, v)| format!("{} : {}", k, v))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Self::Function(fun) => write!(f, "{}", fun),
            Self::Variadic(v) => write!(f, "Variadic({})", v.as_ref()),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name.clone() {
            write!(f, "{} : {}", name, self.kind)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join(", ");
        let returns = self
            .returns
            .iter()
            .map(|r| format!("{}", r))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "Function {}({}){}",
            self.name.clone().unwrap_or_default(),
            params,
            if returns.is_empty() {
                String::default()
            } else {
                format!(" -> {}", returns)
            }
        )
    }
}

impl Class {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.enums.is_empty() && self.methods.is_empty()
    }
    pub fn get_base(s: &str) -> Option<&str> {
        s.rfind('.').map(|pos| &s[..pos])
    }
    pub fn get_end(s: &str) -> Option<&str> {
        s.rfind('.').map(|pos| &s[pos + 1..])
    }
    pub fn show(&self) -> String {
        format!(
            "Class {}\n{}\n{}\n{}",
            self.name,
            self.enums
                .iter()
                .map(|e| format!("  {}", Enum::show(e)))
                .collect::<Vec<String>>()
                .join("\n"),
            self.fields
                .iter()
                .map(|v| format!("  {}", Var::show(v)))
                .collect::<Vec<String>>()
                .join("\n"),
            self.methods
                .iter()
                .map(|f| format!("  {}", f))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

// impl Def {
//     pub fn show(&self) -> String {
//         match self {
//             Self::Alias(d) => d.show(),
//             Self::Function(d) => format!("{}", d),
//             Self::Class(d) => d.show(),
//             Self::Enum(e) => e.show(),
//         }
//     }
// }
