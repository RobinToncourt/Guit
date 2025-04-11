use std::collections::HashMap;
use std::fmt;
use std::iter::Peekable;
use std::ops::Index;
use std::str::Chars;

use anyhow::anyhow;

pub trait Getter<T> {
    type Output;

    fn get(&self, index: T) -> Option<&Self::Output>;
}

#[derive(Debug, PartialEq)]
pub struct Null;
impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Null")
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Bool(bool),
    Null(Null),
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Object(o) => write!(f, "{o:?}"),
            Value::Array(a) => write!(f, "{a:?}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Null(n) => write!(f, "{n}"),
            Value::Number(n) => write!(f, "{n}"),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Value::Object(object) => &object[index],
            _ => panic!("{self} can't be indexed by {index}."),
        }
    }
}

impl Index<String> for Value {
    type Output = Value;

    fn index(&self, index: String) -> &Self::Output {
        match self {
            Value::Object(object) => &object[index.as_str()],
            _ => panic!("{self} can't be indexed by {index}."),
        }
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Value::Array(array) => &array[index],
            _ => panic!("{self} can't be indexed by {index}."),
        }
    }
}

impl Getter<&str> for Value {
    type Output = Self;

    fn get(&self, index: &str) -> Option<&Self::Output> {
        match self {
            Value::Object(object) => object.get(index),
            _ => None,
        }
    }
}

impl Getter<String> for Value {
    type Output = Self;

    fn get(&self, index: String) -> Option<&Self::Output> {
        match self {
            Value::Object(object) => object.get(&index),
            _ => None,
        }
    }
}

impl Getter<usize> for Value {
    type Output = Self;

    fn get(&self, index: usize) -> Option<&Self::Output> {
        match self {
            Value::Array(array) => array.get(index),
            _ => None,
        }
    }
}

pub fn parse_json(content: &str) -> anyhow::Result<Value> {
    let mut chars = content.chars().peekable();

    if let Some(c) = chars.next() {
        match c {
            '{' => Ok(Value::Object(parse_object(&mut chars)?)),
            '[' => Ok(Value::Array(parse_array(&mut chars)?)),
            '"' => Ok(Value::String(parse_string(&mut chars)?)),
            'f' | 'F' => Ok(Value::Bool(parse_bool(&mut chars, false)?)),
            't' | 'T' => Ok(Value::Bool(parse_bool(&mut chars, true)?)),
            'n' | 'N' => Ok(Value::Null(parse_null(&mut chars)?)),
            '0'..='9' => Ok(Value::Number(parse_number(&mut chars, c)?)),
            _ => Err(anyhow!("Unexpected char {c}.")),
        }
    } else {
        Ok(Value::Null(Null))
    }
}

fn parse_object(chars: &mut Peekable<Chars>) -> anyhow::Result<HashMap<String, Value>> {
    let mut object: HashMap<String, Value> = HashMap::new();
    let mut key = String::new();
    let mut value = Value::Null(Null);

    let mut is_parsing_left = true;
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if is_parsing_left {
                    key = parse_string(chars)?;
                } else {
                    value = Value::String(parse_string(chars)?);
                }
            }
            ':' => is_parsing_left = false,
            '{' => value = Value::Object(parse_object(chars)?),
            '[' => value = Value::Array(parse_array(chars)?),
            'f' | 'F' => value = Value::Bool(parse_bool(chars, false)?),
            't' | 'T' => value = Value::Bool(parse_bool(chars, true)?),
            'n' | 'N' => value = Value::Null(parse_null(chars)?),
            '0'..='9' => value = Value::Number(parse_number(chars, c)?),
            ',' | '}' => {
                object.insert(key, value);
                key = String::new();
                value = Value::Null(Null);
                is_parsing_left = true;

                if c == '}' {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(object)
}

fn parse_array(chars: &mut Peekable<Chars>) -> anyhow::Result<Vec<Value>> {
    let mut array: Vec<Value> = Vec::new();

    while let Some(c) = chars.next() {
        match c {
            '"' => array.push(Value::String(parse_string(chars)?)),
            '{' => array.push(Value::Object(parse_object(chars)?)),
            '[' => array.push(Value::Array(parse_array(chars)?)),
            'f' | 'F' => array.push(Value::Bool(parse_bool(chars, false)?)),
            't' | 'T' => array.push(Value::Bool(parse_bool(chars, true)?)),
            'n' | 'N' => array.push(Value::Null(parse_null(chars)?)),
            '0'..='9' => array.push(Value::Number(parse_number(chars, c)?)),
            ']' => break,
            _ => {}
        }
    }

    Ok(array)
}

fn parse_string(chars: &mut Peekable<Chars>) -> anyhow::Result<String> {
    let mut last_char: Option<char> = None;
    let mut string_buffer = String::new();

    for c in chars {
        if c == '"' {
            if let Some(lc) = last_char {
                if lc == '\\' {
                    last_char = Some(c);
                    string_buffer.push(c);
                } else {
                    return Ok(string_buffer);
                }
            } else {
                return Ok(string_buffer);
            }
        } else {
            last_char = Some(c);
            string_buffer.push(c);
        }
    }

    Err(anyhow!("Unexpected end of json."))
}

fn parse_bool(chars: &mut Peekable<Chars>, is_true: bool) -> anyhow::Result<bool> {
    let bool_end = if is_true { "rue" } else { "alse" };

    let bool_end_str: String = chars.take(bool_end.len()).collect();
    if bool_end_str.to_lowercase() == bool_end {
        Ok(is_true)
    } else {
        Err(anyhow!("Unexpected end of json."))
    }
}

fn parse_null(chars: &mut Peekable<Chars>) -> anyhow::Result<Null> {
    let ull: String = chars.take(3).collect();

    if ull.to_lowercase() == "ull" {
        Ok(Null)
    } else {
        Err(anyhow!("Unexpected end of json."))
    }
}

fn parse_number(chars: &mut Peekable<Chars>, first: char) -> anyhow::Result<f64> {
    let mut string_buffer = first.to_string();
    while let Some(c) = chars.peek() {
        match c {
            '0'..='9' | '.' => string_buffer.push(chars.next().unwrap()),
            _ => break,
        }
    }

    Ok(string_buffer.parse::<f64>()?)
}

#[cfg(test)]
mod json_test {
    use super::*;

    const TEXT: &str = r#"{
    "glossary": {
        "title": "example glossary",
        "GlossDiv": {
            "title": "S",
            "GlossList": {
                "GlossEntry": {
                    "ID": "SGML",
                    "SortAs": "SGML",
                    "GlossTerm": "Standard Generalized Markup Language",
                    "Acronym": "SGML",
                    "Abbrev": "ISO 8879:1986",
                    "GlossDef": {
                        "para": "A meta-markup language, used to create markup languages such as DocBook.",
                        "GlossSeeAlso": ["GML", "XML"]
                    },
                "GlossSee": "markup",
                "SomeNull": null
                }
            }
        }
    },
"autre": "rien",
"nothing": "",
"guillemet": "\""
}"#;

    #[test]
    fn test_getter() {
        let json = parse_json(TEXT).unwrap();

        assert!(json.get("glossary").is_some());
    }

    #[test]
    fn test_parse_string() {
        let normal = r#"normal": rest of the json..."#;
        let empty = r#"": rest of the json..."#;
        // let guillemet = r#"abc\def": rest of the json..."#;

        assert_eq!(
            parse_string(&mut normal.chars().peekable()).unwrap(),
            r#"normal"#.to_string()
        );
        assert_eq!(
            parse_string(&mut empty.chars().peekable()).unwrap(),
            "".to_string()
        );
        // assert_eq!(parse_string(guillemet.chars()), r#"abc"def"#.to_string());
    }

    #[test]
    fn test_parse_bool() {
        let fale = "false";
        let tru = "true";
        let neither = "none";

        let mut fale_chars = fale.chars().peekable();
        fale_chars.next();
        let mut tru_chars = tru.chars().peekable();
        tru_chars.next();
        let mut neither_chars = neither.chars().peekable();
        neither_chars.next();

        assert_eq!(parse_bool(&mut fale_chars, false).unwrap(), false);
        assert_eq!(parse_bool(&mut tru_chars, true).unwrap(), true);
        assert!(parse_bool(&mut neither_chars, false).is_err());
    }

    #[test]
    fn test_parse_null() {
        let null = "NULL";
        let not_null = "not null";

        let mut null_chars = null.chars().peekable();
        null_chars.next();
        let mut not_null_chars = not_null.chars().peekable();
        not_null_chars.next();

        assert_eq!(parse_null(&mut null_chars).unwrap(), Null);
        assert!(parse_null(&mut not_null_chars).is_err());
    }

    #[test]
    fn test_parse_number() {
        let integer = "42, rest of the json...";
        let float = "42.12, rest of the json...";

        let mut integer_chars = integer.chars().peekable();
        integer_chars.next();
        let mut float_chars = float.chars().peekable();
        float_chars.next();

        assert_eq!(parse_number(&mut integer_chars, '4').unwrap(), 42_f64);
        assert_eq!(parse_number(&mut float_chars, '4').unwrap(), 42.12_f64);
    }
}
