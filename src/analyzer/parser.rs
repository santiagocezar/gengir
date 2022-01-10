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
    pub fn missing_attribute(attr: &str, position: TextPosition) -> Self {
        Self {
            pos: position,
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

pub struct Attributes(Vec<OwnedAttribute>, TextPosition);

impl Attributes {
    pub fn get(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|attr| attr.name.local_name == String::from(name))
            .and_then(|attr| Some(attr.value.to_owned()))
    }

    pub fn get_must(&self, name: &str) -> Result<String> {
        self.get(name).ok_or(Error::missing_attribute(name, self.1))
    }
}

pub struct Event {
    pub event: XmlEvent,
    pub tree: EventReader<Box<dyn Read>>,
    pub depth: usize,
}

#[macro_export]
macro_rules! tag_matches {
    ($ev:expr, $($tag:expr),*) => {
        if let Some(r) = $ev.matches_tag([$($tag,)*]) {
            r
        } else { return Ok(None) }
    };
}

impl Event {
    pub fn check_start(&self, tag: &str) -> Option<Attributes> {
        if let XmlEvent::StartElement {
            name, attributes, ..
        } = &self.event
        {
            if name.local_name == tag {
                return Some(Attributes(attributes.to_owned(), self.tree.position()));
            }
        }
        None
    }

    pub fn matches_tag<'b>(
        &self,
        tags: impl IntoIterator<Item = &'b str>,
    ) -> Option<(usize, Attributes, &'b str)> {
        for tag in tags {
            if let Some(attrs) = self.check_start(tag) {
                return Some((self.depth, attrs, tag));
            }
        }
        None
    }

    pub fn below(&mut self, depth: usize) -> Result<bool> {
        self.event = self.tree.next()?;

        match &self.event {
            XmlEvent::StartElement { .. } => self.depth += 1,
            XmlEvent::EndElement { .. } => self.depth -= 1,
            XmlEvent::EndDocument => self.depth = 0,
            _ => (),
        };
        Ok(self.depth >= depth)
    }

    pub fn consume<T, F>(mut tree: EventReader<Box<dyn Read>>, mut func: F) -> TagResult<T>
    where
        F: FnMut(&mut Event) -> TagResult<T>,
    {
        let xml_event = tree.next()?;
        let mut ev = &mut Event {
            event: xml_event,
            tree,
            depth: 1,
        };

        while ev.below(1)? {
            if let Some(v) = func(&mut ev)? {
                return Ok(Some(v));
            }
        }

        Ok(None)
    }
}
