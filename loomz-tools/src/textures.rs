use std::process::Command;
use std::path::{Path, PathBuf, Component};
use super::{Error, match_filter};

fn compressonator_in_path() -> bool {
    Command::new("compressonatorcli").output().is_ok()
}

fn texture_output_path(path: &PathBuf) -> PathBuf {
    let mut output = PathBuf::from("assets/textures/");

    let mut root_found = false;
    for component in path.components() {
        if let Component::Normal(name) = component {
            if root_found {
                output.push(name);
            } else {
                root_found |= name == "textures";
            }
        }
    }

    output.set_extension("ktx2");

    output
}

fn compress_single_texture(input_path: &Path, output_path: &Path, compression_format: &str) -> Result<(), Error> {
    println!("Compressing {:?} to {:?} using {}", input_path, output_path, compression_format);
    let mut cmd = Command::new("compressonatorcli");

    cmd.arg("-fd")
        .arg(compression_format)
        .arg(input_path)
        .arg(output_path)
        .arg("-silent")
        .arg("-noprogress");

    if compression_format == "BC7" {
        cmd.arg("-Quality").arg("0.5"); // 1 for best quality, 0.05 for the worst. HAS A MASSIVE IMPACT ON COMPRESSION TIME
    }
    
    cmd.output()?;

    Ok(())
}

pub fn compile_textures(filters: &Vec<String>) {
    if !compressonator_in_path() {
        println!("compressonatorcli not found in path");
        return;
    }

    for entry in glob::glob("./assets/dev/textures/*.png").unwrap().filter_map(Result::ok) {
        if !match_filter(&entry, filters) {
            continue;
        }
        
        let out_path = texture_output_path(&entry);
        compress_single_texture(&entry, &out_path, "BC7").unwrap();
    }
}
