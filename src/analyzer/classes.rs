use std::io::Read;

use crate::declarations::{Class, Function, FunctionKind, Param, Type};

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

    pub fn try_an_class(&mut self, ev: &mut Event) -> TagResult<Class> {
        ev.try_analyzing([CLASS_TAG, INTERFACE_TAG, RECORD_TAG], |ev, tag, attrs| {
            let name = safe_name(attrs.get_must("name")?);
            let mut bases = Vec::new();
            let mut fields = Vec::new();
            let mut methods = Vec::new();
            let mut doc = None;

            if let Some(parent) = attrs.get("parent") {
                let typ = class_or_type_to_native(&parent);
                if let Type::ExternalClass { module, .. } = &typ {
                    self.imports.insert(module.clone());
                }
                bases.push(typ)
            }

            ev.until_closes(tag, |ev| {
                if doc.is_none() {
                    doc = self.try_an_doc(ev)?;
                }

                if let Some(f) = self.try_an_member(ev)? {
                    fields.push(f)
                }
                if let Some(i) = self.try_an_implementor(ev)? {
                    bases.push(i)
                }
                if let Some(m) = self.try_an_function(ev, Some(name.clone()))? {
                    if m.name == "new" {
                        let mut constructor = m.clone();
                        constructor.name = "__init__".into();
                        constructor.return_type = Type::Primitive("None".into());
                        constructor.kind = FunctionKind::Method;
                        // PyGObject constructors parameters are keyword-only
                        constructor.parameters.insert(0, Param::Star);
                        constructor.parameters.insert(0, Param::Instance);
                        methods.push(constructor)
                    }
                    methods.push(m)
                }

                Ok(false)
            })?;

            return Ok(Some(Class {
                name,
                bases,
                doc,
                fields,
                methods,
            }));
        })
    }
}
