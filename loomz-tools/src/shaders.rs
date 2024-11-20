use std::path::{Path, PathBuf};
use std::process::Command;
use super::{Error, err, match_filter};

fn glslang_in_path() -> bool {
    Command::new("glslang").output().is_ok()
}

fn compile_shader(input: &Path, output: &Path) -> Result<(), Error> {
    println!("Compiling {:?} to {:?}", input, output);
    let output = Command::new("glslang")
        .arg("-V100")
        .arg("-o")
        .arg(output)
        .arg(input)
        .output()?;

    if output.status.code() == Some(2) {
        let output = String::from_utf8(output.stdout).unwrap();
        return Err(err(output));
    }

    Ok(())
}

pub fn compile_shaders(filters: &Vec<String>) {
    if !glslang_in_path() {
        eprintln!("WARNING: glslang not found in PATH. Skipping shader compilation");
        return;
    }

    for entry in glob::glob("./assets/shaders/*.glsl").unwrap().filter_map(Result::ok) {
        if !match_filter(&entry, filters) {
            continue;
        }

        let output_name = entry.file_name().and_then(|f| f.to_str() ).unwrap();
        let mut output = PathBuf::from("./assets/shaders");
        output.push(output_name);
        output.set_extension("spv");

        if let Err(e) = compile_shader(&entry, &output) {
            println!("Failed to compile shader: {}", e);
        }
    }
}
