use crate::{
    declarations::{Enumeration, Var},
    tag_matches,
};

use super::{
    common::safe_name,
    parser::{Event, TagResult},
    Analyzer,
};

const BITFIELD_TAG: &str = "bitfield";
const ENUMERATION_TAG: &str = "enumeration";
const MEMBER_TAG: &str = "member";

impl Analyzer {
    fn try_an_enum_bitfield_member(&self, ev: &mut Event) -> TagResult<Var> {
        self.try_an_variable(MEMBER_TAG, ev).map(|o| {
            o.map(|mut v| {
                v.name = safe_name(v.name.to_uppercase());
                v
            })
        })
    }

    pub fn try_an_enum(&self, ev: &mut Event) -> TagResult<Enumeration> {
        let (depth, attrs, ..) = tag_matches!(ev, ENUMERATION_TAG, BITFIELD_TAG);

        let name = attrs.get_must("name")?;
        let mut values = Vec::new();
        let mut doc = None;

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?
            }
            if let Some(mut member) = self.try_an_enum_bitfield_member(ev)? {
                member.name = safe_name(member.name.to_uppercase());
                values.push(member);
            }
        }

        Ok(Some(Enumeration { name, doc, values }))
    }
}
