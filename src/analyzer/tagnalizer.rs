use std::{io::Read, result};

pub use xml::reader::XmlEvent;
use xml::{
    attribute::OwnedAttribute,
    common::{Position, TextPosition},
    EventReader,
};

#[derive(Debug)]
pub enum ErrorKind {
    Xml(xml::reader::Error),
    MissingAttribute(String),
}
#[derive(Debug)]
pub struct Error {
    pub pos: TextPosition,
    pub kind: ErrorKind,
}
impl Error {
    pub fn missing_attribute<R: Read>(ev: &Event<R>, attr: &str) -> Self {
        Self {
            pos: ev.tree.position(),
            kind: ErrorKind::MissingAttribute(attr.into()),
        }
    }
}
impl From<xml::reader::Error> for Error {
    fn from(err: xml::reader::Error) -> Self {
        Self {
            pos: err.position(),
            kind: ErrorKind::Xml(err),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
pub type TagResult<T> = Result<Option<T>>;

pub struct Attributes(pub Vec<OwnedAttribute>);

impl Attributes {
    pub fn get(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|attr| attr.name.local_name == String::from(name))
            .and_then(|attr| Some(attr.value.to_owned()))
    }
}

pub struct Event<'a, R: Read> {
    pub event: XmlEvent,
    pub tree: &'a mut EventReader<R>,
    pub attrs: Attributes,
}

impl<'a, R: Read> Event<'a, R> {
    pub fn check_start(&mut self, tag: &str) -> Option<Attributes> {
        if let XmlEvent::StartElement {
            name, attributes, ..
        } = &self.event
        {
            if name.local_name == tag {
                return Some(Attributes(attributes.to_owned()));
            }
        }
        None
    }

    pub fn check_start_for(
        &mut self,
        tags: impl IntoIterator<Item = &'a str>,
    ) -> Option<(&'a str, Attributes)> {
        for tag in tags.into_iter() {
            if let Some(attrs) = self.check_start(tag) {
                return Some((tag, attrs));
            }
        }
        None
    }

    /// Loops `func` until it returns true or it reaches the end of the element
    /// tagged `tag`.
    pub fn analyze_tag<F>(&mut self, tag: &str, attrs: Attributes, mut func: F) -> Result<()>
    where
        F: FnMut(&mut Event<R>) -> Result<bool>,
    {
        let mut done = false;
        self.attrs = attrs;
        loop {
            self.event = self.tree.next()?;
            match &self.event {
                XmlEvent::EndElement { name } => {
                    if name.local_name == tag {
                        break;
                    }
                }
                XmlEvent::EndDocument => {
                    break;
                }
                _ => {
                    if !done {
                        done = func(self)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks if the event matches the start of element `tag`, and if so, loops
    /// `func` until the end of the element.
    pub fn try_analyze_tag<T, F>(&mut self, tag: &str, mut func: F) -> TagResult<T>
    where
        F: FnMut(&mut Event<R>) -> TagResult<T>,
    {
        if let Some(attrs) = self.check_start(tag) {
            let mut result: Option<T> = None;
            self.analyze_tag(tag, attrs, |ev| {
                result = func(ev)?;
                Ok(result.is_some())
            })?;
            Ok(result)
        } else {
            Ok(None)
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<String> {
        self.attrs.get(name)
    }

    pub fn get_attr_must(&self, name: &str) -> Result<String> {
        self.get_attr(name)
            .ok_or(Error::missing_attribute(self, name))
    }
}

pub fn start_analyzing<T, R, F>(tree: &mut EventReader<R>, mut func: F) -> TagResult<T>
where
    R: Read,
    F: FnMut(&mut Event<R>) -> Result<Option<T>>,
{
    loop {
        let e = tree.next()?;

        match e {
            XmlEvent::EndDocument => {
                break;
            }
            event => {
                if let Some(v) = func(&mut Event {
                    event,
                    tree,
                    attrs: Attributes(Vec::new()),
                })? {
                    return Ok(Some(v));
                }
            }
        }
    }
    Ok(None)
}
