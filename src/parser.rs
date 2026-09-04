#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(warnings)]

use crate::ast::{FoamDict, FoamElement, FoamValue};
use std::fmt;

#[derive(Debug)]
pub struct FoamParseError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for FoamParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for FoamParseError {}

/// Parse an OpenFOAM dictionary string into a `FoamDict` AST.
pub fn parse_foam_dict(input: &str) -> Result<FoamDict, FoamParseError> {
    let mut dict = FoamDict::new();
    let mut chars = input.chars().peekable();
    let mut line_num = 1;

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            if c == '\n' {
                line_num += 1;
            }

            chars.next();
            continue;
        }

        if c == '/' {
            chars.next();

            match chars.peek() {
                Some('/') => {
                    chars.next();
                    let mut comment = String::from("//");

                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            line_num += 1;
                            break;
                        }

                        comment.push(ch);
                    }

                    dict.elements.push(FoamElement::Comment(comment));
                    continue;
                }

                Some('*') => {
                    chars.next();
                    let mut comment = String::from("/*");
                    let mut is_banner = false;

                    for ch in chars.by_ref() {
                        comment.push(ch);

                        if ch == '\n' {
                            line_num += 1;
                        }

                        if comment.contains("OpenFOAM: The Open Source CFD Toolbox") {
                            is_banner = true;
                        }

                        if comment.ends_with("*/") {
                            break;
                        }
                    }

                    if is_banner {
                        dict.elements.push(FoamElement::HeaderBanner(comment));
                    } else {
                        dict.elements.push(FoamElement::Comment(comment));
                    }

                    continue;
                }

                _ => {
                    return Err(FoamParseError {
                        message: "Unexpected '/' character".to_string(),
                        line: line_num,
                    });
                }
            }
        }

        if c == '#' {
            let mut directive = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    break;
                }

                directive.push(ch);
                chars.next();
            }

            skip_whitespace(&mut chars, &mut line_num);

            let mut val_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == ';' || ch == '\n' {
                    if ch == ';' {
                        chars.next();
                    }

                    break;
                }

                val_str.push(ch);
                chars.next();
            }

            dict.elements.push(FoamElement::Directive {
                name: directive,
                value: val_str.trim().to_string(),
            });

            continue;
        }

        if c == '$' {
            let mut macro_name = String::new();
            chars.next();

            while let Some(&ch) = chars.peek() {
                if ch == ';' || ch.is_whitespace() {
                    if ch == ';' {
                        chars.next();
                    }

                    break;
                }

                macro_name.push(ch);
                chars.next();
            }

            dict.elements.push(FoamElement::MacroRef(macro_name));
            continue;
        }

        let mut name = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == '{' || ch == ';' || ch == '(' {
                break;
            }

            name.push(ch);
            chars.next();
        }

        if name.is_empty() {
            chars.next();
            continue;
        }

        skip_whitespace(&mut chars, &mut line_num);

        if let Some(&'{') = chars.peek() {
            chars.next();

            let inner_dict = parse_subdict(&mut chars, &mut line_num)?;

            let has_semicolon = if let Some(&';') = chars.peek() {
                chars.next();
                true
            } else {
                false
            };

            dict.elements.push(FoamElement::Block {
                name,
                dict: inner_dict,
                has_semicolon,
            });

            continue;
        }

        let mut raw_val = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == ';' {
                chars.next();
                break;
            }

            if ch == '\n' {
                line_num += 1;
            }

            raw_val.push(ch);
            chars.next();
        }

        let val_str = raw_val.trim();
        let value = parse_value_str(val_str);

        dict.elements.push(FoamElement::Entry { key: name, value });
    }

    Ok(dict)
}

fn parse_subdict(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line_num: &mut usize,
) -> Result<FoamDict, FoamParseError> {
    let mut dict = FoamDict::new();

    while let Some(&c) = chars.peek() {
        if c == '}' {
            chars.next();
            return Ok(dict);
        }

        if c.is_whitespace() {
            if c == '\n' {
                *line_num += 1;
            }

            chars.next();
            continue;
        }

        if c == '/' {
            chars.next();

            match chars.peek() {
                Some('/') => {
                    chars.next();
                    let mut comment = String::from("//");

                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            *line_num += 1;
                            break;
                        }

                        comment.push(ch);
                    }

                    dict.elements.push(FoamElement::Comment(comment));
                    continue;
                }

                Some('*') => {
                    chars.next();
                    let mut comment = String::from("/*");

                    for ch in chars.by_ref() {
                        comment.push(ch);

                        if ch == '\n' {
                            *line_num += 1;
                        }

                        if comment.ends_with("*/") {
                            break;
                        }
                    }

                    dict.elements.push(FoamElement::Comment(comment));
                    continue;
                }

                _ => {}
            }
        }

        let mut name = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == '{' || ch == ';' || ch == '}' {
                break;
            }

            name.push(ch);
            chars.next();
        }

        if name.is_empty() {
            continue;
        }

        skip_whitespace(chars, line_num);

        if let Some(&'{') = chars.peek() {
            chars.next();

            let inner = parse_subdict(chars, line_num)?;

            let has_semicolon = if let Some(&';') = chars.peek() {
                chars.next();
                true
            } else {
                false
            };

            dict.elements.push(FoamElement::Block {
                name,
                dict: inner,
                has_semicolon,
            });

            continue;
        }

        let mut raw_val = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == ';' {
                chars.next();
                break;
            }

            if ch == '\n' {
                *line_num += 1;
            }

            raw_val.push(ch);
            chars.next();
        }

        let val_str = raw_val.trim();
        let value = parse_value_str(val_str);

        dict.elements.push(FoamElement::Entry { key: name, value });
    }

    Ok(dict)
}

fn skip_whitespace(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    line_num: &mut usize,
) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            if ch == '\n' {
                *line_num += 1;
            }

            chars.next();
        } else {
            break;
        }
    }
}

fn parse_value_str(s: &str) -> FoamValue {
    let s_clean = s.trim();

    if s_clean.is_empty() {
        return FoamValue::String(String::new());
    }

    if s_clean == "true" || s_clean == "on" || s_clean == "yes" {
        return FoamValue::Bool(true);
    }

    if s_clean == "false" || s_clean == "off" || s_clean == "no" {
        return FoamValue::Bool(false);
    }

    if let Ok(i) = s_clean.parse::<i64>() {
        return FoamValue::Int(i);
    }

    if let Ok(f) = s_clean.parse::<f64>() {
        return FoamValue::Scalar(f);
    }

    if s_clean.starts_with('(') && s_clean.ends_with(')') {
        let inner = s_clean[1..s_clean.len() - 1].trim();
        let parts: Vec<&str> = inner.split_whitespace().collect();

        if let Some(numbers) = parts
            .iter()
            .map(|p| p.parse::<f64>())
            .collect::<Result<Vec<f64>, _>>()
            .ok()
            .filter(|n| n.len() == 3)
        {
            return FoamValue::Vector(numbers);
        }
    }

    if s_clean.starts_with('[') && s_clean.ends_with(']') {
        let inner = s_clean[1..s_clean.len() - 1].trim();
        let parts: Vec<&str> = inner.split_whitespace().collect();

        if let Ok(dims) = parts
            .iter()
            .map(|p| p.parse::<i32>())
            .collect::<Result<Vec<i32>, _>>()
        {
            return FoamValue::DimensionSet(dims);
        }
    }

    if let Some(inner) =
        s_clean.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
    {
        return FoamValue::String(inner.to_string());
    }

    if let Some(rest) = s_clean.strip_prefix('$') {
        return FoamValue::MacroRef(rest.to_string());
    }

    FoamValue::String(s_clean.to_string())
}
