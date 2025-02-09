mod shared;
mod shaders;
mod textures;
mod generate_sprite;
mod generate_font;

use std::path::Path;

type Error = Box<dyn ::std::error::Error>;

fn filters() -> Option<Vec<String>> {
    let index = ::std::env::args().position(|arg| arg.as_str() == "-f" || arg.as_str() == "--filters" )?;
    let filters = ::std::env::args().skip(index + 1).next()?;
    Some(filters.split(',').map(|v| v.to_string() ).collect())
}

fn match_filter(entry: &Path, filters: &[String]) -> bool {
    if filters.is_empty() {
        return true;
    }

    let entry_str = entry.to_str().unwrap_or("");
    filters.iter().any(|f| entry_str.matches(f).next().is_some() )
}

fn must_watch() -> bool {
    ::std::env::args().any(|arg| arg.as_str() == "--watch")
}

fn execute_command() -> Option<String> {
    let position = ::std::env::args().position(|arg| arg.as_str() == "-c");
    position.and_then(|p| ::std::env::args().skip(p+1).next() )
}

fn watch() {
    unimplemented!("Watching changes is not implemented");
}

/// Removes all .DS_Store files from the directory when I move stuff from my macbook
fn remove_ds_store() {  
    for entry in glob::glob("./**/.DS_Store").unwrap().filter_map(Result::ok) {
        println!("Removing {:?}", entry);
        ::std::fs::remove_file(&entry).unwrap();
    }
}

/// Move texture atlas json files
fn move_atlas_json(filters: &Vec<String>) {
    for entry in glob::glob("./assets/dev/textures/*.json").unwrap().filter_map(Result::ok) {
        if !match_filter(&entry, filters) {
            continue;
        }

        let file_name = entry.file_name().and_then(|name| name.to_str() ).unwrap_or("");

        let mut dst = ::std::path::PathBuf::new();
        dst.push("assets");
        dst.push(file_name);

        println!("Copying {:?} to {:?}", &entry, &dst);

        if dst.exists() {
            if let Err(e) = ::std::fs::remove_file(&dst) {
                eprintln!("Failed to remove {:?}: {}", dst, e);
            }
        }

        if let Err(e) = ::std::fs::copy(&entry, &dst) {
            eprintln!("Failed to copy {:?} to {:?}: {}", &entry, dst, e);
        }
    }
}

fn once_off() {
    let filters = filters().unwrap_or_default();
    shaders::compile_shaders(&filters);
    textures::compile_textures(&filters);
    move_atlas_json(&filters);
}

fn main() {
    if let Some(cmd) = execute_command() {
        let filters = filters().unwrap_or_default();

        match cmd.as_str() {
            "remove_ds_store" => remove_ds_store(),
            "generate_sprites" => {
                generate_sprite::generate_sprites(&filters);
                textures::compile_textures(&filters);
                move_atlas_json(&filters);
            },
            "generate_fonts" => {
                generate_font::generate_font(&filters);
            },
            value => { println!("Unknown command: {:?}", value); }
        }
    } else {
        match must_watch() {
            true => watch(),
            false => once_off()
        }
    }
}

//
// Error
//

#[derive(Debug)]
struct ErrorString(String);

fn err<V: Into<String>>(msg: V) -> Box<ErrorString> {
    Box::new(ErrorString(msg.into()))
}

impl ::std::fmt::Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ::std::error::Error for ErrorString {
}
