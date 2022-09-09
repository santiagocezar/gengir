mod classes;
mod common;
mod enumerations;
mod functions;
mod parser;
mod types;

use indexmap::{IndexMap, IndexSet};
use std::{
    collections::HashSet,
    fmt::format,
    fs::File,
    io::{BufReader, Read},
    mem,
    path::{Path, PathBuf},
};

use xml::{EventReader, ParserConfig};

use crate::{
    declarations::{Class, Namespace, Param, Type, Var},
    tag_matches,
};

use self::{
    common::safe_name,
    parser::{Event, TagResult},
};

const GIR_PATH: &str = "/usr/share/gir-1.0/";
const INCLUDE_TAG: &str = "include";
const REPOSITORY_TAG: &str = "repository";
const NAMESPACE_TAG: &str = "namespace";
const CONSTANT_TAG: &str = "constant";

/// Analyzes a gir document
pub struct Analyzer {
    ignore_docs: bool,
    pub depth: usize,
    pub namespaces: IndexSet<Namespace>,
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
            depth: 0,
            namespaces: IndexSet::new(),
        }
    }

    pub fn analyze_repository(&mut self, module: &str, version: &str) {
        eprintln!("{}{} v{}", &"| ".repeat(self.depth), module, version);
        self.depth += 1;
        let fname = format!("{}-{}.gir", module, version);
        let gir = File::open(GIR_PATH.to_string() + &fname).unwrap();
        // let file = BufReader::new(file);

        let ns = self.analyze(gir);
        self.depth -= 1;

        self.namespaces.insert(ns);
        // let mut analyzer = Analyzer::new(no_docs);
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

    fn try_a_repository(&mut self, ev: &mut Event) -> TagResult<Namespace> {
        let (depth, ..) = tag_matches!(ev, REPOSITORY_TAG);

        let mut imports = HashSet::new();
        let mut namespace = None;

        while ev.below(depth)? {
            if let Some((name, version)) = self.try_an_include(ev)? {
                if !self.namespaces.contains(name.as_str()) {
                    self.analyze_repository(&name, &version);
                }
                imports.insert(name);
            }
            if let Some(mut ns) = self.try_a_namespace(ev)? {
                ns.imports.extend(imports.drain());
                namespace = Some(ns)
            }
        }

        Ok(namespace)
    }

    fn try_a_namespace(&mut self, ev: &mut Event) -> TagResult<Namespace> {
        let (depth, attrs, ..) = tag_matches!(ev, NAMESPACE_TAG);

        let name = attrs.get_must("name")?;
        let mut imports = HashSet::new();
        let mut constants = Vec::new();
        let mut enums = Vec::new();
        let mut functions = Vec::new();
        let mut classes = IndexMap::<String, Class>::new();

        // Add constructor parameters from parent class
        // TODO: Add connect signals too

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
            if let Some((c, i)) = self.try_an_class(ev)? {
                imports.extend(i);
                classes.insert(c.name.clone(), c);
            }
        }

        // store the keys
        let keys: Vec<_> = classes.keys().cloned().collect();

        for key in keys {
            // take the class
            let class = classes.remove(&key);
            if let Some(mut class) = class {
                for base in &class.bases {
                    // try to get the actual class
                    let resolved = match base {
                        Type::ExternalClass { module, name } => self
                            .namespaces
                            .get(module.as_str())
                            .map(|m| m.classes.get(name.as_str()))
                            .flatten(),
                        Type::LocalClass(name) => classes.get(name.as_str()),
                        _ => None,
                    };

                    if let Some(r) = resolved {
                        for param in &r.constructor.parameters {
                            if matches!(param, Param::Named { .. }) {
                                // copy only the named parameters
                                class.constructor.parameters.push(param.clone())
                            }
                        }
                    }
                }
                classes.insert(key, class);
            }
        }

        // Sort class order of appearance by parents (topology sort)

        let mut sorted_classes = IndexSet::with_capacity(classes.len());

        while let Some((_, c)) = classes.pop() {
            traverse(&mut classes, &mut sorted_classes, c);
        }

        /*TODO:
        check if this works

        while let Some((key, c)) = classes.pop() {
            for base in &c.bases {
                if let Type::LocalClass(base) = base {
                    if let Some(base) = h.remove(base) {
                        traverse(h, s, base);
                    }
                }
            }
            traverse(&mut classes, &mut sorted_classes, c);
        }
        */

        Ok(Some(Namespace {
            name,
            imports,
            constants,
            enums,
            functions,
            classes: sorted_classes,
        }))
    }

    pub fn try_an_include(&mut self, ev: &mut Event) -> TagResult<(String, String)> {
        let (_, attrs, ..) = tag_matches!(ev, INCLUDE_TAG);
        let name = attrs.get_must("name")?;
        let version = attrs.get("version");
        Ok(version.map(|ver| (name, ver)))
    }

    /// Parses and consumes the source, returns the resulting [`Namespace`]
    pub fn analyze(&mut self, source: impl Read + 'static) -> Namespace {
        let config = ParserConfig::new().trim_whitespace(true);
        let tree = EventReader::new_with_config(Box::new(source) as Box<dyn Read>, config);

        Event::consume(tree, |e| self.try_a_repository(e))
            .unwrap()
            .unwrap()
    }
}
