use super::{
    tagnalizer::{Event, TagResult, XmlEvent},
    types::{try_an_type, Type},
};
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Read;

const DOC_TAG: &str = "doc";
const MEMBER_TAG: &str = "member";

#[derive(Debug)]
pub enum Value {
    Number(String),
    Str(String),
    None,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

pub fn analyze_value(val: &str) -> Value {
    lazy_static! {
        static ref INT: Regex = Regex::new("^[-+]?[0-9]+$").unwrap();
    }
    if INT.is_match(val) {
        Value::Number(val.to_string())
    } else if val == "(null)" {
        Value::None
    } else {
        Value::Str(val.to_string())
    }
}

pub fn try_an_doc<R: Read>(ev: &mut Event<R>) -> TagResult<String> {
    ev.try_analyze_tag(DOC_TAG, |ev| match &ev.event {
        XmlEvent::Characters(text) => Ok(Some(text.to_owned())),
        _ => Ok(None),
    })
}

#[derive(Debug)]
pub struct Var {
    pub name: String,
    pub value: Option<Value>,
    pub typ: Option<&'static Type>,
    pub doc: Option<String>,
    pub constant: bool,
}

pub fn try_an_member<R: Read>(ev: &mut Event<R>) -> TagResult<Var> {
    if let Some(attrs) = ev.check_start(MEMBER_TAG) {
        let name = ev.get_attr_must("name")?;
        let value = attrs.get("value").map(|s| analyze_value(&s));
        let mut typ = None;
        let mut doc = None;
        ev.analyze_tag(MEMBER_TAG, attrs, |ev| {
            if doc.is_none() {
                doc = try_an_doc(ev)?;
            }
            if typ.is_none() {
                typ = try_an_type(ev)?;
            }
            Ok(doc.is_some() && typ.is_some())
        })?;
        return Ok(Some(Var {
            name,
            value,
            typ,
            doc,
            constant: false,
        }));
    }
    Ok(None)
}
