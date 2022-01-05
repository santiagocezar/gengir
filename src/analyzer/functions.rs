use std::io::Read;

use crate::declarations::{Function, FunctionKind, Param, Type};

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
const SIGNAL_TAG: &str = "signal";

impl Analyzer {
    fn try_an_return_value(&mut self, ev: &mut Event) -> TagResult<(Option<String>, Type)> {
        ev.try_analyzing([RETURN_VALUE_TAG], |ev, tag, attrs| {
            let mut doc = None;
            let mut typ = Type::Any;
            ev.until_closes(tag, |ev| {
                if doc.is_none() {
                    doc = self.try_an_doc(ev)?;
                }
                if matches!(typ, Type::Any) {
                    if let Some(t) = self.try_an_class_type(ev)? {
                        typ = t;
                    }
                }
                Ok(doc.is_some() && !matches!(typ, Type::Any))
            })?;
            Ok(Some((doc, typ)))
        })
    }

    fn try_an_param(&mut self, ev: &mut Event) -> TagResult<Param> {
        ev.try_analyzing([PARAMETER_TAG, INSTANCE_PARAMETER_TAG], |ev, tag, attrs| {
            if tag == INSTANCE_PARAMETER_TAG {
                return Ok(Some(Param::Instance));
            }

            let name = safe_name(attrs.get_must("name")?);
            let variadic = name == "...";
            let name = if variadic { String::from("args") } else { name };

            let optional = attrs.get("nullable").map(|n| n == "1").unwrap_or(false);
            let mut doc = None;
            let mut typ = Type::Any;

            ev.until_closes(tag, |ev| {
                if doc.is_none() {
                    doc = self.try_an_doc(ev)?;
                }
                if matches!(typ, Type::Any) {
                    if let Some(t) = self.try_an_class_type(ev)? {
                        typ = t;
                    }
                }
                Ok(doc.is_some() && !matches!(typ, Type::Any))
            })?;

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
        })
    }

    pub fn try_an_function(
        &mut self,
        ev: &mut Event,
        method_of: Option<String>,
    ) -> TagResult<Function> {
        ev.try_analyzing(
            [
                FUNCTION_TAG,
                METHOD_TAG,
                VIRTUAL_METHOD_TAG,
                CONSTRUCTOR_TAG,
                SIGNAL_TAG,
            ],
            |ev, tag, attrs| {
                let name = safe_name(attrs.get_must("name")?);
                let mut parameters = Vec::new();
                let mut doc = None;
                let mut kind = match tag {
                    FUNCTION_TAG => FunctionKind::Static,
                    SIGNAL_TAG => FunctionKind::Signal,
                    METHOD_TAG | VIRTUAL_METHOD_TAG | CONSTRUCTOR_TAG => FunctionKind::Method,
                    _ => unreachable!(),
                };
                let mut return_doc = None;
                let mut return_type = Type::Any;

                if let Some(class) = method_of {
                    if tag == CONSTRUCTOR_TAG {}
                }

                ev.until_closes(tag, |ev| {
                    if doc.is_none() {
                        doc = self.try_an_doc(ev)?;
                    }
                    if matches!(return_type, Type::Any) && return_doc.is_none() {
                        if let Some((rd, rt)) = self.try_an_return_value(ev)? {
                            return_doc = rd;
                            return_type = rt;
                        }
                    }
                    ev.simple_analyze("parameters", |ev, _| {
                        if let Some(p) = self.try_an_param(ev)? {
                            parameters.push(p);
                        }
                        Ok(None as Option<()>)
                    })?;

                    Ok(doc.is_some()
                        && parameters.len() > 0
                        && !matches!(return_type, Type::Any)
                        && return_doc.is_some())
                })?;

                // check if the method is static by checking if a self parameter exists
                if matches!(kind, FunctionKind::Method)
                    && (parameters.is_empty()
                        || (parameters.len() > 0 && !matches!(parameters[0], Param::Instance)))
                {
                    kind = FunctionKind::StaticMethod
                }

                return Ok(Some(Function {
                    name,
                    parameters,
                    return_type,
                    kind,
                    return_doc,
                    doc,
                }));
            },
        )
    }
}
