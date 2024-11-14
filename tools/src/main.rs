use std::path::{Path, PathBuf};
use std::process::Command;

type Error = Box<dyn ::std::error::Error>;

fn compressonator_in_path() -> bool {
    Command::new("compressonatorcli").output().is_ok()
}

fn glslang_in_path() -> bool {
    Command::new("glslang").output().is_ok()
}


fn must_watch() -> bool {
    ::std::env::args().any(|arg| arg.as_str() == "--watch")
}

fn execute_command() -> Option<String> {
    let position = ::std::env::args().position(|arg| arg.as_str() == "-c");
    position.and_then(|p| ::std::env::args().skip(p+1).next() )
}

fn watch() {
    unimplemented!();
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
        return Err(Box::new(ErrorString(output)));
    }

    Ok(())
}

fn compile_shaders() {
    for entry in glob::glob("./assets/shaders/*.glsl").unwrap().filter_map(Result::ok) {
        let output_name = entry.file_name().and_then(|f| f.to_str() ).unwrap();
        let mut output = PathBuf::from("./assets/shaders");
        output.push(output_name);
        output.set_extension("spv");

        if let Err(e) = compile_shader(&entry, &output) {
            println!("Failed to compile shader: {}", e);
        }
    }
}

fn remove_ds_store() {  
    for entry in glob::glob("./assets/dev/**/.DS_Store").unwrap().filter_map(Result::ok) {
        println!("Removing {:?}", entry);
        ::std::fs::remove_file(&entry).unwrap();
    }
}

fn once_off() {
    compile_shaders();
}

fn main() {
    if !compressonator_in_path() {
        eprintln!("WARNING: compressonatorcli not found in PATH");
    }

    if !glslang_in_path() {
        eprintln!("WARNING: glslang not found in PATH");
    }

    if let Some(cmd) = execute_command() {
        match cmd.as_str() {
            "remove_ds_store" => remove_ds_store(),
            value => { println!("{:?}", value); }
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

impl ::std::fmt::Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ::std::error::Error for ErrorString {
}
