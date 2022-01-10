mod classes;
mod common;
mod enumerations;
mod functions;
mod parser;
mod types;

use indexmap::{IndexMap, IndexSet};
use std::{collections::HashSet, io::Read, mem};

use xml::{EventReader, ParserConfig};

use crate::{
    declarations::{Class, Namespace, Type, Var},
    tag_matches,
};

use self::{
    common::safe_name,
    parser::{Event, TagResult},
};

const NAMESPACE_TAG: &str = "namespace";
const CONSTANT_TAG: &str = "constant";

/// Analyzes a gir document
pub struct Analyzer {
    ignore_docs: bool,
    imports: HashSet<String>,
}

fn traverse(h: &mut IndexMap<String, Class>, s: &mut IndexSet<Class>, c: Class) {
    for base in &c.bases {
        if let Type::LocalClass(base) = base {
            if let Some(base) = h.remove(base) {
                traverse(h, s, base);
            }
        }
    }
    s.insert(c);
}

impl Analyzer {
    pub fn new(ignore_docs: bool) -> Self {
        Self {
            ignore_docs,
            imports: HashSet::new(),
        }
    }

    fn try_an_constant(&self, ev: &mut Event) -> TagResult<Var> {
        self.try_an_variable(CONSTANT_TAG, ev).map(|o| {
            o.map(|mut v| {
                v.name = safe_name(v.name);
                v.constant = true;
                v
            })
        })
    }

    fn try_an_namespace(&mut self, ev: &mut Event) -> TagResult<Namespace> {
        let (depth, attrs, ..) = tag_matches!(ev, NAMESPACE_TAG);

        let name = attrs.get_must("name")?;
        let mut constants = Vec::new();
        let mut enums = Vec::new();
        let mut functions = Vec::new();
        let mut classes = IndexMap::<String, Class>::new();

        while ev.below(depth)? {
            if let Some(e) = self.try_an_enum(ev)? {
                enums.push(e);
            }
            if let Some(c) = self.try_an_constant(ev)? {
                constants.push(c);
            }
            if let Some(f) = self.try_an_function(ev, None)? {
                functions.push(f);
            }
            if let Some(c) = self.try_an_class(ev)? {
                classes.insert(c.name.clone(), c);
            }
        }

        // Sort class order of appearance by parents (topology sort)

        let mut sorted_classes = IndexSet::with_capacity(classes.len());

        while !classes.is_empty() {
            let next_key = classes.first().map(|(k, _)| k.clone());

            if let Some(key) = next_key {
                if let Some(c) = classes.remove(&key) {
                    traverse(&mut classes, &mut sorted_classes, c);
                }
            }
        }

        Ok(Some(Namespace {
            name,
            imports: mem::replace(&mut self.imports, HashSet::new()),
            constants,
            enums,
            functions,
            classes: sorted_classes,
        }))
    }

    /// Parses and consumes the source, returns the resulting [`Namespace`]
    pub fn analyze(&mut self, source: impl Read + 'static) -> Namespace {
        let config = ParserConfig::new().trim_whitespace(true);
        let tree = EventReader::new_with_config(Box::new(source) as Box<dyn Read>, config);

        Event::consume(tree, |e| self.try_an_namespace(e))
            .unwrap()
            .unwrap()
    }
}
