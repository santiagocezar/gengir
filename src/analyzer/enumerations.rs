use super::{
    common::{try_an_doc, try_an_member, Var},
    tagnalizer::{Error, Event, TagResult},
};
use std::io::Read;

const ENUMERATION_TAG: &str = "enumeration";

#[derive(Debug)]
pub struct Enumeration {
    pub name: String,
    pub values: Vec<Var>,
    pub doc: Option<String>,
}

pub fn try_an_enum<R: Read>(ev: &mut Event<R>) -> TagResult<Enumeration> {
    if let Some(attrs) = ev.check_start(ENUMERATION_TAG) {
        let name = ev.get_attr_must("name")?;
        let mut values = Vec::new();
        let mut doc = None;
        ev.analyze_tag(ENUMERATION_TAG, attrs, |ev| {
            if doc.is_none() {
                doc = try_an_doc(ev)?
            }
            if let Some(member) = try_an_member(ev)? {
                values.push(member);
            }
            Ok(false)
        })?;
        return Ok(Some(Enumeration { name, doc, values }));
    }
    Ok(None)
}
