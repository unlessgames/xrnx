use clap::Parser as ClapParser;
use std::{
    fs::*,
    io::{Read, Write},
    path::PathBuf,
};

mod error;
mod json;
mod json_to_types;
mod library;
mod lua_parser;
mod render;
mod sources;
mod types;

use library::Library;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "PATH")]
    library: PathBuf,
    output: PathBuf,
}
fn replace_inside(file_path: &str, from: &str, to: &str) -> Result<(), error::Error> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut file = File::create(file_path)?;
    file.write_all(content.replace(from, to).as_bytes())?;
    Ok(())
}
fn main() -> Result<(), error::Error> {
    let args = Args::parse();
    let lib = Library::from_path(args.library.clone())?;
    let docs = lib.export_docs();
    let out = args.output.to_string_lossy();

    std::fs::create_dir_all(format!("{}/API", out))?;
    for (name, content) in docs.iter() {
        let mut file = File::create(format!("{}/API/{}.md", out, name))?;
        file.write_all(content.as_bytes())?;
    }
    replace_inside(
        &format!("{}/SUMMARY.md", out),
        "<!-- API -->",
        &docs
            .iter()
            .map(|(name, _)| format!("  - [{}](API/{}.md)", name.clone(), name.clone()))
            .collect::<Vec<String>>()
            .join("\n"),
    )?;
    Ok(())
}
