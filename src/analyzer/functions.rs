use crate::{
    declarations::{Function, FunctionKind, Param, Type},
    tag_matches,
};

use super::{
    common::safe_name,
    parser::{Event, TagResult},
    Analyzer,
};

const PARAMETER_TAG: &str = "parameter";
const INSTANCE_PARAMETER_TAG: &str = "instance-parameter";

const PARAMETERS_TAG: &str = "parameters";
const RETURN_VALUE_TAG: &str = "return-value";

const FUNCTION_TAG: &str = "function";
const METHOD_TAG: &str = "method";
const VIRTUAL_METHOD_TAG: &str = "virtual-method";
const CONSTRUCTOR_TAG: &str = "constructor";

impl Analyzer {
    fn try_an_return_value(&mut self, ev: &mut Event) -> TagResult<(Option<String>, Type)> {
        let (depth, ..) = tag_matches!(ev, RETURN_VALUE_TAG);

        let mut doc = None;
        let mut typ = Type::Any;

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?;
            }
            if matches!(typ, Type::Any) {
                if let Some(t) = self.try_an_class_type(ev)? {
                    typ = t;
                }
            }
        }

        Ok(Some((doc, typ)))
    }

    fn try_an_param(&mut self, ev: &mut Event) -> TagResult<Param> {
        let (depth, attrs, tag) = tag_matches!(ev, PARAMETER_TAG, INSTANCE_PARAMETER_TAG);

        if tag == INSTANCE_PARAMETER_TAG {
            return Ok(Some(Param::Instance));
        }

        let name = safe_name(attrs.get_must("name")?);
        let variadic = name == "...";
        let name = if variadic { String::from("args") } else { name };

        let optional = attrs.get("nullable").map(|n| n == "1").unwrap_or(false);
        let mut doc = None;
        let mut typ = Type::Any;

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?;
            }
            if matches!(typ, Type::Any) {
                if let Some(t) = self.try_an_class_type(ev)? {
                    typ = t;
                }
            }
        }

        if variadic {
            Ok(Some(Param::Variadic { name, typ, doc }))
        } else {
            Ok(Some(Param::Named {
                name,
                typ,
                doc,
                optional,
            }))
        }
    }

    pub fn try_an_function(
        &mut self,
        ev: &mut Event,
        method_of: Option<String>,
    ) -> TagResult<Function> {
        let (depth, attrs, tag) = tag_matches!(
            ev,
            FUNCTION_TAG,
            METHOD_TAG,
            VIRTUAL_METHOD_TAG,
            CONSTRUCTOR_TAG
        );

        let name = safe_name(attrs.get_must("name")?);
        let mut parameters = Vec::new();
        let mut doc = None;
        let mut kind = match tag {
            FUNCTION_TAG => FunctionKind::Static,
            METHOD_TAG | CONSTRUCTOR_TAG => FunctionKind::Method,
            VIRTUAL_METHOD_TAG => FunctionKind::Virtual,
            _ => unreachable!(),
        };
        let mut return_doc = None;
        let mut return_type = Type::Any;

        if let Some(class) = method_of {
            if tag == CONSTRUCTOR_TAG {
                return_type = Type::LocalClass(class)
            }
        }

        while ev.below(depth)? {
            if doc.is_none() {
                doc = self.try_an_doc(ev)?;
            }
            if matches!(return_type, Type::Any) && return_doc.is_none() {
                if let Some((rd, rt)) = self.try_an_return_value(ev)? {
                    return_doc = rd;
                    return_type = rt;
                }
            }

            if let Some((depth, ..)) = ev.matches_tag([PARAMETERS_TAG]) {
                while ev.below(depth)? {
                    if let Some(p) = self.try_an_param(ev)? {
                        parameters.push(p);
                    }
                }
            }
        }

        // check if the method is static by checking if a self parameter exists
        let first_param = if parameters.len() > 0 {
            Some(&parameters[0])
        } else {
            None
        };

        kind = match (kind, first_param) {
            (FunctionKind::Method, Some(Param::Instance)) => FunctionKind::Method,
            (FunctionKind::Method, _) => FunctionKind::StaticMethod,
            (kind, _) => kind,
        };

        return Ok(Some(Function {
            name,
            parameters,
            return_type,
            kind,
            return_doc,
            doc,
        }));
    }
}
