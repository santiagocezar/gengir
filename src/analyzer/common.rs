use crate::{
    declarations::{Value, Var},
    tag_matches,
};

use super::{
    parser::{Event, TagResult, XmlEvent},
    Analyzer,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

const DOC_TAG: &str = "doc";
const MEMBER_TAG: &str = "member";

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

pub fn safe_name(mut name: String) -> String {
    lazy_static! {
        static ref KEYWORDS: HashSet<&'static str> = HashSet::from([
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield", "print"
        ]);
    }
    if name.chars().next().map(|c| c.is_numeric()).unwrap_or(true)
        || KEYWORDS.contains(name.as_str())
    {
        name.insert(0, '_');
        name
    } else {
        name
    }
    .replace('-', "_")
}

impl Analyzer {
    pub fn try_an_doc(&self, ev: &mut Event) -> TagResult<String> {
        if self.ignore_docs {
            return Ok(None);
        }
        let (depth, ..) = tag_matches!(ev, DOC_TAG);
        while ev.below(depth)? {
            if let XmlEvent::Characters(text) = &ev.event {
                return Ok(Some(text.to_owned()));
            }
        }
        Ok(None)
    }

    pub fn try_an_variable(&self, tag: &str, ev: &mut Event) -> TagResult<Var> {
        let (depth, attrs, ..) = tag_matches!(ev, tag);

        let name = attrs.get_must("name")?;
        let value = attrs.get("value").map(|s| analyze_value(&s));
        let mut typ = None;
        let mut doc = None;

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?;
            }
            if typ.is_none() {
                typ = self.try_an_type(ev)?;
            }
        }

        Ok(Some(Var {
            name,
            value,
            typ,
            doc,
            constant: false,
        }))
    }

    pub fn try_an_member(&self, ev: &mut Event) -> TagResult<Var> {
        self.try_an_variable(MEMBER_TAG, ev).map(|o| {
            o.map(|mut v| {
                v.name = safe_name(v.name);
                v
            })
        })
    }
}
