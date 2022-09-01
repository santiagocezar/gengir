use crate::{declarations::Type, tag_matches};
use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{
    parser::{Event, TagResult},
    Analyzer,
};

macro_rules! map {
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
            map!(gboolean => bool),
            map!(gint => int),
            map!(guint => int),
            map!(gint8 => int),
            map!(guint8 => int),
            map!(gint16 => int),
            map!(guint16 => int),
            map!(gint32 => int),
            map!(guint32 => int),
            map!(gint64 => int),
            map!(guint64 => int),
            map!(gsize => int),
            map!(gpointer => Any),
            map!(none => None),
            map!(gchar => str),
            map!(guchar => str),
            map!(lit "gchar*" => str),
            map!(lit "guchar*" => str),
            map!(glong => long),
            map!(gulong => long),
            map!(glong64 => long),
            map!(gulong64 => long),
            map!(gshort => int),
            map!(gushort => int),
            map!(gshort64 => int),
            map!(gushort64 => int),
            map!(gfloat => float),
            map!(gdouble => float),
            map!(string => str),
            map!(GString => str),
            map!(utf8 => str),
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

    pub fn try_an_type_like_tag(&self, tag: &str, ev: &mut Event) -> TagResult<Type> {
        let (depth, attrs, ..) = tag_matches!(ev, tag);

        while ev.below(depth)? {}

        Ok(Some(
            attrs
                .get("name")
                .map_or(Type::Any, |s| class_or_type_to_native(&s)),
        ))
    }

    pub fn try_a_class_type(&self, ev: &mut Event) -> TagResult<Type> {
        self.try_an_type_like_tag(TYPE_TAG, ev)
    }
}
