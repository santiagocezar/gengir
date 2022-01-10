use crate::{declarations::Type, tag_matches};
use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{
    parser::{Event, TagResult},
    Analyzer,
};

macro_rules! typ {
    (lit $glib:tt => $py:ident) => {
        ($glib, Type::Primitive(String::from(stringify!($py))))
    };
    ($glib:ident => $py:ident) => {
        (
            stringify!($glib),
            Type::Primitive(String::from(stringify!($py))),
        )
    };
    ($glib:ident => Any) => {
        (stringify!($glib), Type::Any)
    };
}

const ANY: Type = Type::Any;

pub fn glib_to_native_type(type_name: &str) -> &'static Type {
    lazy_static! {
        static ref MAP: HashMap<&'static str, Type> = HashMap::from([
            typ!(gboolean => bool),
            typ!(gint => int),
            typ!(guint => int),
            typ!(gint8 => int),
            typ!(guint8 => int),
            typ!(gint16 => int),
            typ!(guint16 => int),
            typ!(gint32 => int),
            typ!(guint32 => int),
            typ!(gint64 => int),
            typ!(guint64 => int),
            typ!(gsize => int),
            typ!(gpointer => Any),
            typ!(none => None),
            typ!(gchar => str),
            typ!(guchar => str),
            typ!(lit "gchar*" => str),
            typ!(lit "guchar*" => str),
            typ!(glong => long),
            typ!(gulong => long),
            typ!(glong64 => long),
            typ!(gulong64 => long),
            typ!(gshort => int),
            typ!(gushort => int),
            typ!(gshort64 => int),
            typ!(gushort64 => int),
            typ!(gfloat => float),
            typ!(gdouble => float),
            typ!(string => str),
            typ!(GString => str),
            typ!(utf8 => str),
        ]);
    };
    MAP.get(type_name).unwrap_or(&ANY)
}

pub fn class_or_type_to_native(type_name: &str) -> Type {
    let t = glib_to_native_type(type_name);
    match t {
        &Type::Any => {
            if let Some((module, name)) = type_name.split_once('.') {
                Type::ExternalClass {
                    module: module.into(),
                    name: name.into(),
                }
            } else {
                Type::LocalClass(type_name.into())
            }
        }
        typ => typ.clone(),
    }
}

const TYPE_TAG: &str = "type";

impl Analyzer {
    pub fn try_an_type(&self, ev: &mut Event) -> TagResult<&'static Type> {
        let (depth, attrs, ..) = tag_matches!(ev, TYPE_TAG);

        while ev.below(depth)? {}

        Ok(Some(
            attrs.get("name").map_or(&ANY, |s| glib_to_native_type(&s)),
        ))
    }

    pub fn try_an_type_like_tag(&mut self, tag: &str, ev: &mut Event) -> TagResult<Type> {
        let (depth, attrs, ..) = tag_matches!(ev, tag);

        while ev.below(depth)? {}

        Ok(Some(attrs.get("name").map_or(Type::Any, |s| {
            let typ = class_or_type_to_native(&s);
            if let Type::ExternalClass { module, .. } = &typ {
                self.imports.insert(module.clone());
            }
            typ
        })))
    }

    pub fn try_an_class_type(&mut self, ev: &mut Event) -> TagResult<Type> {
        self.try_an_type_like_tag(TYPE_TAG, ev)
    }
}
