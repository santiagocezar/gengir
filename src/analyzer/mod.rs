mod common;
mod enumerations;
mod functions;
mod tagnalizer;
mod types;

use std::{collections::HashSet, io::Read};

use xml::{EventReader, ParserConfig};

use crate::analyzer::common::Value;

use self::{
    common::Var,
    enumerations::{try_an_enum, Enumeration},
    functions::{try_an_function, Function},
    tagnalizer::{start_analyzing, Event, TagResult},
};

pub struct Class {
    /*
pub name: String,
pub bases: Vec<String>,
pub fields: Vec<Var>,
pub methods: Vec<Function>,
pub doc: Option<String>,*/}
const NAMESPACE_TAG: &str = "namespace";

pub struct Namespace {
    pub enums: Vec<Enumeration>,
    pub functions: Vec<Function>,
}

pub fn try_an_namespace<R: Read>(ev: &mut Event<R>) -> TagResult<Namespace> {
    if let Some(attrs) = ev.check_start(NAMESPACE_TAG) {
        let mut enums = Vec::new();
        let mut functions = Vec::new();
        ev.analyze_tag(NAMESPACE_TAG, attrs, |ev| {
            if let Some(e) = try_an_enum(ev)? {
                println!("got an enum!");
                enums.push(e);
            }
            if let Some(f) = try_an_function(ev)? {
                println!("got an function!");
                functions.push(f);
            }
            Ok(false)
        })?;
        return Ok(Some(Namespace { enums, functions }));
    }
    Ok(None)
}

pub struct Analyzer<R: Read> {
    pub ignore_docs: bool,
    pub source: Option<R>,

    pub gi_imports: HashSet<String>,
    pub classes: Vec<Class>,
}

impl<R: Read> Analyzer<R> {
    pub fn new(source: R, ignore_docs: bool) -> Self {
        Self {
            ignore_docs,
            source: Some(source),
            gi_imports: HashSet::new(),
            classes: Vec::new(),
        }
    }

    pub fn analyze(&mut self) {
        let config = ParserConfig::new().trim_whitespace(true);
        let mut parser = EventReader::new_with_config(self.source.take().unwrap(), config);

        let ns = start_analyzing(&mut parser, try_an_namespace)
            .unwrap()
            .unwrap();

        for e in ns.enums {
            println!("found enum \"{}\"", e.name);
            println!("with values:");
            for v in e.values {
                let value = v.value.unwrap_or(Value::Str("was_rust_None".into()));
                println!("  {} = {}", v.name, value);
            }
        }
        for f in ns.functions {
            print!("{}(", f.name);

            for (i, p) in f.parameters.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}: {}", p.name, p.typ);
            }
            println!(") -> {}", f.return_type);
        }
    }
}
