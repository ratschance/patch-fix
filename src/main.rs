#[macro_use] extern crate clap;
extern crate glob;
#[macro_use] extern crate lazy_static;
extern crate regex;

mod patch;

use glob::glob;
use patch::*;
use std::path::PathBuf;

fn main() {
    let matches = clap_app!(patch_fix =>
        (version: "v0.0.1")
        (author: "Conrad Ratschan")
        (about: "Fixes patches that cannot be applied with git-am")
        (@arg signed: -s --signed +takes_value "Name and email for a 'signed off by' line")
        (@arg patch: +required "Patch or directory with patches to apply")
        (@arg repo: +required "Sets the Git repository to apply commits on")
    ).get_matches();

    let mut patch_file = PathBuf::from(matches.value_of("patch").unwrap());
    let mut repo = PathBuf::from(matches.value_of("repo").unwrap());
    let signed = matches.value_of("signed").unwrap_or("");

    if patch_file.is_dir() {
        patch_file.push("*.patch");
        match enumerate_patches(&patch_file) {
            Some(patches) => {
                for patch in patches {
                    println!("{}", patch.message);
                }
            }
            None => {
                println!("Unable to continue");
                return; 
            }
        }
    }
}

fn enumerate_patches(dir: &PathBuf) -> Option<Vec<Patch>> {
    let mut patches: Vec<Patch> = Vec::new();
    for entry in glob(dir.to_str().unwrap()).expect("Failed to read glob") {
        match entry {
            Ok(path_buf) => {
                match parse_patch(path_buf.as_path()) {
                    Some(patch) => patches.push(patch),
                    None => return None,
                }
            }
            Err(e) => {
                println!("{:?}", e);
                return None;
            }
        }
    }
    Some(patches)
}
