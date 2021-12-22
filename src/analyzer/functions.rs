use std::{io::Read, ptr::NonNull};

use super::{
    common::try_an_doc,
    tagnalizer::{Event, TagResult},
    types::{try_an_class, try_an_type, Type},
};

pub struct Param {
    pub name: String,
    pub typ: Type,
    pub doc: Option<String>,
    pub optional: bool,
    pub variadic: bool,
}

pub enum FunctionKind {
    Static,
    Method,
    StaticMethod,
}

pub struct Function {
    pub name: String,
    pub parameters: Vec<Param>,
    pub return_type: Type,
    pub kind: FunctionKind,
    pub return_doc: Option<String>,
    pub doc: Option<String>,
}

const PARAMETER_TAG: &str = "parameter";
const INSTANCE_PARAMETER_TAG: &str = "instance-parameter";

const PARAMETERS_TAG: &str = "parameters";
const RETURN_VALUE_TAG: &str = "return-value";

const FUNCTION_TAG: &str = "function";
const METHOD_TAG: &str = "method";
const VIRTUAL_METHOD_TAG: &str = "virtual-method";
const CONSTRUCTOR_TAG: &str = "constructor";

fn try_an_return_value<R: Read>(ev: &mut Event<R>) -> TagResult<(Option<String>, Type)> {
    if let Some(attrs) = ev.check_start(RETURN_VALUE_TAG) {
        let mut doc = None;
        let mut typ = Type::Any;
        ev.analyze_tag(RETURN_VALUE_TAG, attrs, |ev| {
            if doc.is_none() {
                doc = try_an_doc(ev)?;
            }
            if matches!(typ, Type::Any) {
                if let Some(t) = try_an_class(ev)? {
                    typ = t;
                }
            }
            Ok(doc.is_some() && !matches!(typ, Type::Any))
        })?;
        return Ok(Some((doc, typ)));
    }
    Ok(None)
}

fn try_an_param<R: Read>(ev: &mut Event<R>) -> TagResult<Param> {
    if let Some((tag, attrs)) = ev.check_start_for([PARAMETER_TAG, INSTANCE_PARAMETER_TAG]) {
        let name = if tag == INSTANCE_PARAMETER_TAG {
            String::from("self")
        } else {
            ev.get_attr_must("name")?
        };
        let variadic = name == "...";
        let optional = attrs.get("nullable").map(|n| n == "1").unwrap_or(false);
        let mut doc = None;
        let mut typ = Type::Any;

        ev.analyze_tag(tag, attrs, |ev| {
            if doc.is_none() {
                doc = try_an_doc(ev)?;
            }
            println!("fun {:?} {:?} {:?}", doc, typ, ev.event);
            if matches!(typ, Type::Any) {
                if let Some(t) = try_an_class(ev)? {
                    typ = t;
                }
            }
            Ok(doc.is_some() && !matches!(typ, Type::Any))
        })?;
        return Ok(Some(Param {
            name,
            typ,
            doc,
            optional,
            variadic,
        }));
    }
    Ok(None)
}

pub fn try_an_function<R: Read>(ev: &mut Event<R>) -> TagResult<Function> {
    if let Some((tag, attrs)) = ev.check_start_for([
        FUNCTION_TAG,
        METHOD_TAG,
        VIRTUAL_METHOD_TAG,
        CONSTRUCTOR_TAG,
    ]) {
        let name = ev.get_attr_must("name")?;
        let mut parameters = Vec::new();
        let mut doc = None;
        let mut kind = match tag {
            FUNCTION_TAG => FunctionKind::Static,
            METHOD_TAG | VIRTUAL_METHOD_TAG | CONSTRUCTOR_TAG => FunctionKind::Method,
            _ => unreachable!(),
        };
        let mut return_doc = None;
        let mut return_type = Type::Any;

        ev.analyze_tag(tag, attrs, |ev| {
            if doc.is_none() {
                doc = try_an_doc(ev)?;
            }
            if matches!(return_type, Type::Any) && return_doc.is_none() {
                if let Some((rd, rt)) = try_an_return_value(ev)? {
                    return_doc = rd;
                    return_type = rt;
                }
            }
            ev.try_analyze_tag("parameters", |ev| {
                if let Some(p) = try_an_param(ev)? {
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
            && (parameters.is_empty() || (parameters.len() > 0 && parameters[0].name != "self"))
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
    }
    Ok(None)
}
