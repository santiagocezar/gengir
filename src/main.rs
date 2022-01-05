mod analyzer;
mod declarations;
mod pygen;
mod xml_skip;

use std::{
    ffi::OsString,
    fs::{self, read_dir, File},
    io::{self, BufReader, BufWriter, Write},
    os::unix::prelude::OsStringExt,
    path::PathBuf,
    process::Command,
};

use analyzer::{Analyzer, Namespace};
use clap::Parser;

use pygen::PythonGenerator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Files to use as input for the generator. If not provided it uses all files in /usr/share/gir-1.0/
    gir_files: Vec<PathBuf>,

    // Directory to store the package typings. $site-packages/gi-stubs by default
    #[clap(short, long, parse(from_os_str))]
    out_dir: Option<PathBuf>,

    /// GTK version to generate typings for
    #[clap(long = "gtk", default_value_t = 3)]
    gtk_version: u8,

    /// Exclude docstrings in the typings
    #[clap(short, long)]
    no_docs: bool,
}

fn matching_gtk_version(gtk: u8, path: &PathBuf) -> bool {
    let name = path.file_name().unwrap().to_str().unwrap();
    match (gtk, name) {
        (2, "Gtk-2.0.gir") => true,
        (3, "Gtk-3.0.gir") => true,
        (4, "Gtk-4.0.gir") => true,
        _ if name.starts_with("Gtk-") => false,
        _ => true,
    }
}

fn analyze_path(p: &PathBuf, no_docs: bool) -> Namespace {
    let file = File::open(p).unwrap();
    let file = BufReader::new(file);

    let mut analyzer = Analyzer::new(no_docs);

    println!("parsing {}", p.display());
    analyzer.analyze(file)
    //println!("done parsing {}", p.display());
}

fn create_stub_tree(dir: &PathBuf) -> io::Result<()> {
    let repo = dir.join("repository");

    if let Err(e) = fs::create_dir_all(&repo) {
        match e.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => return Err(e),
        }
    };

    let mut py_typed = File::create(dir.join("py.typed"))?;
    py_typed.write(b"partial\n")?;

    let mut init = File::create(dir.join("__init__.pyi"))?;
    init.write(include_bytes!("gi_init.pyi"))?;

    let mut repo_init = File::create(repo.join("__init__.pyi"))?;
    repo_init.write(b"")?;

    Ok(())
}

fn get_sitepackages() -> PathBuf {
    let out = Command::new("python")
        .arg("-c")
        .arg("import site, os; sp = site.getsitepackages()[0]; print(sp if os.access(sp, os.W_OK | os.X_OK) else site.getusersitepackages(), end='')")
        .output().unwrap();

    PathBuf::from(OsString::from_vec(out.stdout))
}

// "/usr/share/gir-1.0/Gtk-3.0.gir"
fn main() -> io::Result<()> {
    let cli = Args::parse();

    let out_dir = cli
        .out_dir
        .unwrap_or_else(|| get_sitepackages().join("gi-stubs"));

    println!("creating gi-stubs tree in {}", out_dir.display());

    create_stub_tree(&out_dir)?;

    let paths = if cli.gir_files.is_empty() {
        read_dir("/usr/share/gir-1.0/")?
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                if entry.file_type().unwrap().is_file()
                    && path.extension().map(|e| e == "gir").unwrap_or_default()
                    && matching_gtk_version(cli.gtk_version, &path)
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        cli.gir_files
    };

    paths.par_iter().try_for_each(|p| {
        let ns = analyze_path(p, cli.no_docs);

        let filename = p.file_name().map(|s| s.to_string_lossy().to_string());

        if let Some(module) = filename
            .as_ref()
            .map(|f| f.split_once('-'))
            .flatten()
            .map(|(m, _)| m.to_owned() + ".pyi")
        {
            let py = File::create(&out_dir.join("repository").join(module))?;
            let mut buf = BufWriter::new(py);
            let mut gen = PythonGenerator::new(&mut buf);
            gen.write_namespace(ns)?;
        } else {
            println!("couldn't get module name for {}", p.display())
        }
        io::Result::Ok(())
        //generate_module(analyzed, f)
    })?;

    Ok(())
}
