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

pub struct Event<'a> {
    pub event: XmlEvent,
    pub tree: &'a mut EventReader<Box<dyn Read>>,
}

impl<'a> Event<'a> {
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

    /// Loops `func` until it returns true or it reaches the end of the element
    /// tagged `tag`.
    pub fn until_closes<F>(&mut self, tag: &str, mut func: F) -> Result<()>
    where
        F: FnMut(&mut Event) -> Result<bool>,
    {
        let mut done = false;
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
    pub fn try_analyzing<'b, T, F>(
        &mut self,
        tags: impl IntoIterator<Item = &'b str>,
        mut func: F,
    ) -> TagResult<T>
    where
        F: FnOnce(&mut Event, &'b str, Attributes) -> TagResult<T>,
    {
        for tag in tags.into_iter() {
            if let Some(attrs) = self.check_start(tag) {
                return Ok(func(self, tag, attrs)?);
            }
        }
        Ok(None)
    }

    /// Checks if the event matches the start of element `tag`, and if so, loops
    /// `func` until the end of the element.
    pub fn simple_analyze<T, F>(&mut self, tag: &str, mut func: F) -> TagResult<T>
    where
        F: FnMut(&mut Event, &Attributes) -> TagResult<T>,
    {
        self.try_analyzing([tag], |ev, _, attrs| {
            let mut result = func(ev, &attrs)?;

            ev.until_closes(tag, |ev| {
                if result.is_none() {
                    result = func(ev, &attrs)?;
                }
                Ok(result.is_some())
            })?;
            return Ok(result);
        })
    }
}

/*
impl<'a> Iterator for Event<'a> {
    type Item = Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.event = match self.tree.next() {
            Ok(ev) => ev,
            Err(e) => return Some(Err(e.into())),
        };
        match &self.event {
            XmlEvent::EndElement { name } if name.local_name == tag => None,
            XmlEvent::EndDocument => None,
            _ => Some(Ok(())),
        }
    }
}
*/
pub fn start_analyzing<T, F>(tree: &mut EventReader<Box<dyn Read>>, mut func: F) -> TagResult<T>
where
    F: FnMut(&mut Event) -> Result<Option<T>>,
{
    loop {
        let e = tree.next()?;

        match e {
            XmlEvent::EndDocument => {
                break;
            }
            event => {
                if let Some(v) = func(&mut Event { event, tree })? {
                    return Ok(Some(v));
                }
            }
        }
    }
    Ok(None)
}
