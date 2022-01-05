use std::fmt::write;

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(String),
    LocalClass(String),
    ExternalClass { module: String, name: String },
    Any,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(String),
    Str(String),
    None,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub value: Option<Value>,
    pub typ: Option<&'static Type>,
    pub doc: Option<String>,
    pub constant: bool,
}

#[derive(Debug)]
pub struct Enumeration {
    pub name: String,
    pub values: Vec<Var>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Param {
    /// the classic
    Named {
        name: String,
        typ: Type,
        doc: Option<String>,
        optional: bool,
    },
    /// `*args`. these ones cannot be optional
    Variadic {
        name: String,
        typ: Type,
        doc: Option<String>,
    },
    /// in python would be `self`. not called `Self` 'cuz that's a keyword
    Instance,
    /// the `*` that makes the following parameters keyword only
    Star,
}

#[derive(Debug, Clone)]
pub enum FunctionKind {
    Signal,
    Static,
    Method,
    StaticMethod,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Param>,
    pub return_type: Type,
    pub kind: FunctionKind,
    pub return_doc: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub bases: Vec<Type>,
    pub fields: Vec<Var>,
    pub methods: Vec<Function>,
    pub doc: Option<String>,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(s) => write!(f, "{}", s),
            Self::LocalClass(c) => write!(f, "{}", c),
            Self::ExternalClass { module, name } => write!(f, "{}.{}", module, name),
            Self::Any => write!(f, "typing.Any"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(typ) = self.typ {
            if self.constant {
                write!(f, ": typing.Final[{}]", typ)?;
            } else {
                write!(f, ": {}", typ)?;
            }
        }
        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Enumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "enum \"{}\" {{", self.name)?;
        for v in &self.values {
            writeln!(f, "  {}", v)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named {
                name,
                typ,
                optional,
                ..
            } => {
                if *optional {
                    write!(f, "{}?: {}", name, typ)
                } else {
                    write!(f, "{}: {}", name, typ)
                }
            }
            &Self::Variadic { .. } | &Self::Star => write!(f, "..."),
            &Self::Instance => write!(f, "self"),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            FunctionKind::Static => write!(f, "funcion "),
            FunctionKind::Method => write!(f, "method "),
            FunctionKind::StaticMethod => write!(f, "static method "),
            FunctionKind::Signal => write!(f, "signal "),
        }?;

        write!(f, "{}(", self.name)?;

        for (i, p) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", p)?;
        }
        write!(f, ") -> {}", self.return_type)?;
        Ok(())
    }
}
