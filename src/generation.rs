use std::io::{self, Cursor, Write};

use lazy_static::lazy_static;
use regex::Regex;

use crate::declarations::{Class, Enumeration, Function, Namespace, Param, Var};

pub struct PythonGenerator<W: Write> {
    writer: W,
}

#[derive(Clone, Copy)]
struct Indent(usize);

impl std::fmt::Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&"    ".repeat(self.0))
    }
}

impl std::ops::Add<usize> for Indent {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Indent {
    #[inline]
    fn align(&self, text: String) -> String {
        if self.0 == 0 {
            return text;
        }
        text.replace('\n', &(String::from("\n") + &"    ".repeat(self.0)))
    }
}

fn summarize(mut doc: String) -> String {
    lazy_static! {
        static ref PAT: Regex = Regex::new(r"[.]\s").unwrap();
    }
    let period = PAT.find(&doc).map(|s| s.start());
    if let Some(p) = period {
        doc.truncate(p);
    }

    doc.replace('\n', " ")
}

/*

// this code was supposed to generate signals, but it ended up being useless as
// python does not inherit overloads

write!(self.writer, "{i}@typing.overload\n{i}def connect(self: Self, signal: typing.Literal[\"{}\"], callback: typing.Callable[typing.Concatenate[[Self", func.name, i = indent)?;
for (i, p) in func.parameters.drain(..).enumerate() {
    write!(self.writer, ", ")?;
    match p {
        Param::Named { optional, typ, .. } => {
            if optional {
                write!(self.writer, "typing.Optional[{}]", typ)?;
            } else {
                write!(self.writer, "{}", typ)?;
            }
        }
        _ => (),
    }
}
write!(
    self.writer,
    "], P], {}], *bind: P.args, **kwbind: P.kwargs) -> typing.Any:",
    func.return_type
)?;

*/

impl<W: Write> PythonGenerator<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_namespace(&mut self, ns: Namespace) -> io::Result<()> {
        writeln!(self.writer, "import typing")?;
        if ns.enums.len() > 0 {
            writeln!(self.writer, "import enum")?;
        }
        for (i, import) in ns.imports.iter().enumerate() {
            if i == 0 {
                write!(self.writer, "from gi.repository import ")?;
            } else {
                write!(self.writer, ", ")?;
            }
            write!(self.writer, "{}", import)?;
        }
        writeln!(self.writer)?;

        for c in ns.constants {
            self.write_constant(c)?;
        }
        for e in ns.enums {
            self.write_enum(e)?;
        }
        for f in ns.functions {
            self.write_function(f, Indent(0))?;
        }
        for c in ns.classes {
            self.write_class(c)?;
        }

        Ok(())
    }

    fn write_docstring(&mut self, doc: Option<String>, indent: Indent) -> io::Result<bool> {
        if let Some(doc) = doc {
            let doc = indent.align(doc);

            writeln!(self.writer, "{i}\"\"\"\n{i}{}\n{i}\"\"\"", doc, i = indent)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Writes a constant [`Var`] as a python variable definition annotated with
    /// `typing.Final[T]`, and adds an inline comment with the truncated
    /// documentation
    fn write_constant(&mut self, var: Var) -> io::Result<()> {
        write!(self.writer, "{}", var)?;
        if let Some(mut doc) = var.doc {
            doc.find('\n').map(|s| doc.truncate(s));
            write!(self.writer, " # {}", doc)?;
        }
        writeln!(self.writer)
    }

    fn write_enum(&mut self, enumeration: Enumeration) -> io::Result<()> {
        writeln!(self.writer, "class {}(enum.Enum):", enumeration.name)?;

        self.write_docstring(enumeration.doc, Indent(1))?;

        for v in enumeration.values {
            writeln!(self.writer, "    {}", v)?;
            self.write_docstring(v.doc, Indent(1))?;
        }

        Ok(())
    }

    fn write_class(&mut self, class: Class) -> io::Result<()> {
        write!(self.writer, "class {}(", class.name)?;

        for (i, base) in class.bases.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }
            write!(self.writer, "{}", base)?;
        }

        writeln!(self.writer, "):")?;

        let body_indent = Indent(1);
        let mut empty = true;

        empty = empty && !self.write_docstring(class.doc, body_indent)?;

        for field in class.fields {
            writeln!(self.writer, "{}{}", body_indent, field)?;
            empty = false;
        }
        for method in class.methods {
            self.write_function(method, body_indent)?;
            empty = false;
        }

        if empty {
            writeln!(self.writer, "{}...", body_indent)?;
        }

        Ok(())
    }

    fn write_function(&mut self, mut func: Function, indent: Indent) -> io::Result<()> {
        use crate::declarations::FunctionKind::*;

        let body_indent = indent + 1;

        let mut docstring = Cursor::new(Vec::new());

        if let Some(doc) = func.doc {
            docstring.write(summarize(doc).as_bytes())?;
            docstring.write(b"\n\n")?;
        }

        if matches!(func.kind, StaticMethod) {
            writeln!(self.writer, "{}@staticmethod", indent)?;
        }

        write!(self.writer, "{}def {}(", indent, func.name)?;

        for (i, p) in func.parameters.drain(..).enumerate() {
            if i > 0 {
                write!(self.writer, ", ")?;
            }

            let doc = match p {
                Param::Named {
                    name,
                    doc,
                    optional,
                    typ,
                } => {
                    if optional {
                        write!(self.writer, "{}: typing.Optional[{}] = None", name, typ)?;
                    } else {
                        write!(self.writer, "{}: {} = None", name, typ)?;
                    }
                    doc.map(|d| (name, d))
                }
                Param::Variadic { name, doc, typ } => {
                    write!(self.writer, "*{}: {}", name, typ)?;
                    doc.map(|d| (name, d))
                }
                Param::Instance => {
                    write!(self.writer, "self")?;
                    None
                }
                Param::Star => {
                    write!(self.writer, "*")?;
                    None
                }
            };

            if let Some((name, doc)) = doc {
                writeln!(
                    docstring,
                    "{}:param {}: {}",
                    body_indent,
                    name,
                    summarize(doc)
                )?;
            }
        }

        write!(self.writer, ") -> {}:", func.return_type)?;

        if let Some(doc) = func.return_doc {
            writeln!(docstring, "{}:return: {}", body_indent, summarize(doc))?;
        }

        let docstring = docstring.into_inner();

        if docstring.is_empty() {
            writeln!(self.writer, "\n{}...", body_indent)?;
        } else {
            write!(self.writer, "\n{i}\"\"\"", i = body_indent)?;
            self.writer.write(&docstring)?;
            writeln!(self.writer, "{}\"\"\"", body_indent)?;
        }

        Ok(())
    }
}
