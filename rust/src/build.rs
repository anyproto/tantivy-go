extern crate cbindgen;

use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

const FILE_PATH: &str = "../tantivy/bindings.h";

fn add_typedefs() -> io::Result<()> {

    let include = "#include <binding_typedefs.h>\n";

    let mut existing_content = fs::read_to_string(FILE_PATH)?;

    existing_content.insert_str(0, include);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(FILE_PATH)?;

    file.write_all(existing_content.as_bytes())?;
    file.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR")?;
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;

    match cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate() {
        Ok(bindings) => bindings.write_to_file(FILE_PATH),
        Err(e) => return Err(Box::new(e)),
    };

    add_typedefs()?;
    Ok(())
}