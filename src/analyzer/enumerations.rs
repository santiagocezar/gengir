use crate::declarations::{Enumeration, Var};

use super::{
    common::safe_name,
    parser::{Error, Event, TagResult},
    Analyzer,
};
use std::io::Read;

const ENUMERATION_TAG: &str = "enumeration";
const ENUM_VALUE_TAG: &str = "member";

impl Analyzer {
    fn try_an_enum_value(&self, ev: &mut Event) -> TagResult<Var> {
        self.try_an_variable(ENUM_VALUE_TAG, ev).map(|o| {
            o.map(|mut v| {
                v.name = safe_name(v.name.to_uppercase());
                v
            })
        })
    }

    pub fn try_an_enum(&self, ev: &mut Event) -> TagResult<Enumeration> {
        ev.try_analyzing([ENUMERATION_TAG], |ev, tag, attrs| {
            let name = attrs.get_must("name")?;
            let mut values = Vec::new();
            let mut doc = None;
            ev.until_closes(tag, |ev| {
                if doc.is_none() {
                    doc = self.try_an_doc(ev)?
                }
                if let Some(mut member) = self.try_an_enum_value(ev)? {
                    member.name = safe_name(member.name.to_uppercase());
                    values.push(member);
                }
                Ok(false)
            })?;
            Ok(Some(Enumeration { name, doc, values }))
        })
    }
}
