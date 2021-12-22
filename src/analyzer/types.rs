use std::{collections::HashMap, io::Read};

use lazy_static::lazy_static;

use super::tagnalizer::{Event, TagResult};

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(String),
    LocalClass(String),
    ExternalClass { module: String, name: String },
    Any,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(s) => write!(f, "{}", s),
            Self::LocalClass(c) => write!(f, "{}", c),
            Self::ExternalClass { module, name } => write!(f, "{}.{}", module, name),
            Self::Any => write!(f, "any"),
        }
    }
}

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
    ($glib:ident => $mod:ident.$cls:ident) => {
        (
            stringify!($glib),
            Type::ExternalClass {
                module: String::from(stringify!($mod)),
                name: String::from(stringify!($cls)),
            },
        )
    };
}

const ANY: Type = Type::Any;

pub fn glib_to_native_type(type_name: &str) -> &'static Type {
    lazy_static! {
        static ref map: HashMap<&'static str, Type> = HashMap::from([
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
            typ!(gpointer => typing.Any),
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
    map.get(type_name).unwrap_or(&ANY)
}

pub fn class_or_type_to_native(type_name: &str) -> Type {
    let t = glib_to_native_type(type_name);
    println!("{} => {}", type_name, t);
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

pub fn try_an_type<R: Read>(ev: &mut Event<R>) -> TagResult<&'static Type> {
    ev.try_analyze_tag(TYPE_TAG, |ev| {
        let type_name = ev.get_attr_must("name")?;
        Ok(Some(glib_to_native_type(&type_name)))
    })
}

pub fn try_an_class<R: Read>(ev: &mut Event<R>) -> TagResult<Type> {
    ev.try_analyze_tag(TYPE_TAG, |ev| {
        let type_name = ev.get_attr_must("name")?;
        Ok(Some(class_or_type_to_native(&type_name)))
    })
}
