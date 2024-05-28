extern crate cbindgen;

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

fn add_typedefs() -> io::Result<()> {
    let file_path = "go/tantivy/bindings.h";
    let include = "#include <binding_typedefs.h>\n";

    // Чтение содержимого существующего файла
    let mut existing_content = fs::read_to_string(file_path)?;

    // Добавление новой строки в начало содержимого
    existing_content.insert_str(0, include);

    // Открытие файла для записи (перезапись)
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;

    // Запись обновленного содержимого обратно в файл
    file.write_all(existing_content.as_bytes())?;
    file.flush()?;

    Ok(())
}


fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("go/tantivy/bindings.h");

    add_typedefs().unwrap()
}