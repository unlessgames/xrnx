use clap::Parser as ClapParser;
use std::fs::*;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

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
    // Read the file content into a string
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Replace the from string with the to string
    let new_content = content.replace(from, to);

    // Write the modified content back to the file
    let mut file = File::create(file_path)?;
    file.write_all(new_content.as_bytes())?;

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
        &format!(
            "{}",
            docs.iter()
                .map(|(name, _)| format!("  - [{}](API/{}.md)", name.clone(), name.clone()))
                .collect::<Vec<String>>()
                .join("\n")
        ),
    )?;
    Ok(())
}
