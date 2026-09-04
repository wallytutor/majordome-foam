#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(warnings)]

use std::fmt;

/// Represents primitive and structured values in an OpenFOAM dictionary.
#[derive(Debug, Clone, PartialEq)]
pub enum FoamValue {
    Scalar(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Vector(Vec<f64>),
    DimensionSet(Vec<i32>),
    List(Vec<FoamValue>),
    Dict(FoamDict),
    MacroRef(String),
    Raw(String),
}

impl fmt::Display for FoamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::String(v) => {
                if v.contains(' ') || v.contains('|') || v.contains('(') {
                    write!(f, "\"{}\"", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            Self::Bool(v) => write!(f, "{}", if *v { "true" } else { "false" }),
            Self::Vector(vec) => {
                let s = vec
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "({})", s)
            }
            Self::DimensionSet(dims) => {
                let s = dims
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "[{}]", s)
            }
            Self::List(items) => {
                let s = items
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "({})", s)
            }
            Self::Dict(d) => write!(f, "{}", d.to_foam_indent(0)),
            Self::MacroRef(name) => write!(f, "${}", name),
            Self::Raw(raw) => write!(f, "{}", raw),
        }
    }
}

/// Represents individual elements contained in an OpenFOAM dictionary.
#[derive(Debug, Clone, PartialEq)]
pub enum FoamElement {
    Comment(String),
    HeaderBanner(String),
    Directive {
        name: String,
        value: String,
    },
    Entry {
        key: String,
        value: FoamValue,
    },
    Block {
        name: String,
        dict: FoamDict,
        has_semicolon: bool,
    },
    MacroRef(String),
}

/// Abstract Syntax Tree node for an OpenFOAM dictionary file or sub-dict.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FoamDict {
    pub elements: Vec<FoamElement>,
}

impl FoamDict {
    /// Construct a new empty `FoamDict`.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Format dictionary into OpenFOAM text representation with indentation.
    pub fn to_foam_indent(&self, indent_level: usize) -> String {
        let pad = "    ".repeat(indent_level);
        let mut out = String::new();

        for (idx, elem) in self.elements.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }

            match elem {
                FoamElement::Comment(c) => {
                    out.push_str(&format!("{}{}", pad, c));
                }

                FoamElement::HeaderBanner(b) => {
                    out.push_str(b);
                }

                FoamElement::Directive { name, value } => {
                    out.push_str(&format!("{}{} {};", pad, name, value));
                }

                FoamElement::Entry { key, value } => {
                    out.push_str(&format!("{}{} {};", pad, key, value));
                }

                FoamElement::Block {
                    name,
                    dict,
                    has_semicolon,
                } => {
                    out.push_str(&format!("{}{}\n{}{{\n", pad, name, pad));
                    let inner = dict.to_foam_indent(indent_level + 1);

                    if !inner.is_empty() {
                        out.push_str(&inner);
                        out.push('\n');
                    }

                    out.push_str(&format!("{}}}", pad));

                    if *has_semicolon {
                        out.push(';');
                    }
                }

                FoamElement::MacroRef(m) => {
                    out.push_str(&format!("{}{};", pad, m));
                }
            }
        }

        out
    }

    /// Serialize dictionary into canonical OpenFOAM string.
    pub fn to_foam(&self) -> String {
        self.to_foam_indent(0)
    }

    /// Retrieve an entry by slash-separated path (e.g. "solvers/p/tolerance").
    pub fn get_path(&self, key_path: &str) -> Option<FoamValue> {
        let parts: Vec<&str> = key_path.split('/').collect();

        self.get_recursive(&parts)
    }

    fn get_recursive(&self, parts: &[&str]) -> Option<FoamValue> {
        if parts.is_empty() {
            return None;
        }

        let head = parts[0];

        for elem in &self.elements {
            match elem {
                FoamElement::Entry { key, value } if key == head => {
                    if parts.len() == 1 {
                        return Some(value.clone());
                    }
                }

                FoamElement::Block { name, dict, .. } if name == head => {
                    if parts.len() == 1 {
                        return Some(FoamValue::Dict(dict.clone()));
                    }

                    return dict.get_recursive(&parts[1..]);
                }

                _ => {}
            }
        }

        None
    }

    /// Set an entry value by slash-separated path.
    pub fn set_path(&mut self, key_path: &str, value: FoamValue) {
        let parts: Vec<&str> = key_path.split('/').collect();

        self.set_recursive(&parts, value);
    }

    fn set_recursive(&mut self, parts: &[&str], value: FoamValue) {
        if parts.is_empty() {
            return;
        }

        let head = parts[0];

        if parts.len() == 1 {
            for elem in &mut self.elements {
                match elem {
                    FoamElement::Entry { key, value: v } if key == head => {
                        *v = value;
                        return;
                    }
                    _ => {}
                }
            }

            self.elements.push(FoamElement::Entry {
                key: head.to_string(),
                value,
            });

            return;
        }

        for elem in &mut self.elements {
            match elem {
                FoamElement::Block { name, dict, .. } if name == head => {
                    dict.set_recursive(&parts[1..], value);
                    return;
                }
                _ => {}
            }
        }

        let mut new_dict = FoamDict::new();
        new_dict.set_recursive(&parts[1..], value);

        self.elements.push(FoamElement::Block {
            name: head.to_string(),
            dict: new_dict,
            has_semicolon: false,
        });
    }

    /// Remove an entry or sub-block by slash-separated key path.
    pub fn delete_path(&mut self, key_path: &str) -> bool {
        let parts: Vec<&str> = key_path.split('/').collect();

        self.delete_recursive(&parts)
    }

    fn delete_recursive(&mut self, parts: &[&str]) -> bool {
        if parts.is_empty() {
            return false;
        }

        let head = parts[0];

        if parts.len() == 1 {
            let orig_len = self.elements.len();
            self.elements.retain(|e| match e {
                FoamElement::Entry { key, .. } => key != head,
                FoamElement::Block { name, .. } => name != head,
                _ => true,
            });

            return self.elements.len() < orig_len;
        }

        for elem in &mut self.elements {
            match elem {
                FoamElement::Block { name, dict, .. } if name == head => {
                    return dict.delete_recursive(&parts[1..]);
                }
                _ => {}
            }
        }

        false
    }

    /// Add an `#include` directive to the dictionary.
    pub fn add_include(&mut self, filename: &str) {
        self.elements.insert(
            0,
            FoamElement::Directive {
                name: "#include".to_string(),
                value: format!("\"{}\"", filename),
            },
        );
    }

    /// Add an `#includeEtc` directive to the dictionary.
    pub fn add_include_etc(&mut self, filename: &str) {
        self.elements.insert(
            0,
            FoamElement::Directive {
                name: "#includeEtc".to_string(),
                value: format!("\"{}\"", filename),
            },
        );
    }

    /// Return top-level entry/block keys in dictionary order.
    pub fn keys(&self) -> Vec<String> {
        let mut keys = Vec::new();

        for elem in &self.elements {
            match elem {
                FoamElement::Entry { key, .. } => keys.push(key.clone()),
                FoamElement::Block { name, .. } => keys.push(name.clone()),
                _ => {}
            }
        }

        keys
    }
}
