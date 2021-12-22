use std::io::Read;

use xml::{
    reader::{Result, XmlEvent},
    EventReader,
};

pub trait XMLSkip {
    fn skip(&mut self) -> Result<()>;
}

impl<R: Read> XMLSkip for EventReader<R> {
    /// Skips all XML events until the next end tag at the current level.
    ///
    /// Convenience function that is useful for the case where you have
    /// encountered a start tag that is of no interest and want to
    /// skip the entire XML subtree until the corresponding end tag.
    #[inline]
    fn skip(&mut self) -> Result<()> {
        let mut depth = 1;

        while depth > 0 {
            match self.next()? {
                XmlEvent::StartElement { .. } => depth += 1,
                XmlEvent::EndElement { .. } => depth -= 1,
                XmlEvent::EndDocument => unreachable!(),
                _ => {}
            }
        }

        Ok(())
    }
}
