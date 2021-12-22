mod analyzer;
mod xml_skip;

use std::{
    fs::File,
    io::{BufReader, Read},
};

use analyzer::Analyzer;
use xml::{
    reader::{EventReader, XmlEvent},
    ParserConfig,
};

fn main() {
    let file = File::open("/usr/share/gir-1.0/Gtk-3.0.gir").unwrap();
    let file = BufReader::new(file);

    let mut analyzer = Analyzer::new(file, false);

    analyzer.analyze();
}
