use std::collections::HashSet;

use indexmap::IndexSet;

use crate::{
    declarations::{Class, Function, FunctionKind, Param, Type},
    tag_matches, typ,
};

use super::{
    common::safe_name,
    parser::{Event, TagResult},
    types::class_or_type_to_native,
    Analyzer,
};

const CLASS_TAG: &str = "class";
const INTERFACE_TAG: &str = "interface";
const RECORD_TAG: &str = "record";

const IMPLEMENTS_TAG: &str = "implements";

impl Analyzer {
    fn try_an_implementor(&mut self, ev: &mut Event) -> TagResult<Type> {
        self.try_an_type_like_tag(IMPLEMENTS_TAG, ev)
    }

    pub fn try_an_class(&mut self, ev: &mut Event) -> TagResult<(Class, HashSet<String>)> {
        let (depth, attrs, ..) = tag_matches!(ev, CLASS_TAG, INTERFACE_TAG, RECORD_TAG);

        let mut imports = HashSet::new();

        let name = safe_name(attrs.get_must("name")?);
        let mut bases = Vec::new();
        let mut fields = Vec::new();
        let mut methods = IndexSet::new();
        let mut doc = None;

        if let Some(parent) = attrs.get("parent") {
            let typ = class_or_type_to_native(&parent);
            if let Type::ExternalClass { module, .. } = &typ {
                imports.insert(module.clone());
            }
            bases.push(typ)
        }

        let mut con_params = vec![Param::Star];

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?;
            }

            if let Some(f) = self.try_a_property(ev)? {
                con_params.push(Param::Named {
                    name: f.name.clone(),
                    typ: f.typ.unwrap_or(Type::Any),
                    doc: None,
                    optional: true,
                });
                // fields.push(f)
            }
            if let Some(i) = self.try_an_implementor(ev)? {
                if let Type::ExternalClass { module, .. } = &i {
                    imports.insert(module.clone());
                }
                bases.push(i)
            }
            if let Some(m) = self.try_an_function(ev, Some(&name))? {
                methods.insert(m);
            }
        }

        if con_params.len() == 1 {
            con_params.clear();
        }

        let constructor = Function {
            name: "__init__".into(),
            parameters: con_params,
            return_type: typ!(None),
            kind: FunctionKind::Method,
            return_doc: None,
            doc: None,
        };

        return Ok(Some((
            Class {
                name,
                bases,
                doc,
                fields,
                constructor,
                methods,
            },
            imports,
        )));
    }
}
