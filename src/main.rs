#[macro_use] extern crate clap;
extern crate glob;
#[macro_use] extern crate lazy_static;
extern crate regex;

mod patch;

use glob::glob;
use patch::*;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let matches = clap_app!(patch_fix =>
        (version: "0.1.0")
        (author: "Conrad Ratschan")
        (about: "Fixes patches that cannot be applied with git-am")
        (@arg signed: -s --signed +takes_value "Name and email for a 'signed off by' line")
        (@arg patch: +required "Patch or directory with patches to apply")
    ).get_matches();

    let mut patch_file = PathBuf::from(matches.value_of("patch").unwrap());
    let signed = matches.value_of("signed").unwrap_or("");

    if patch_file.is_dir() {
        patch_file.push("*.patch");
        match enumerate_patches(&patch_file) {
            Some(patches) => {
                apply_patches(&patches);
            }
            None => {
                println!("Unable to continue");
                return; 
            }
        }
    }
}

fn apply_patches(patches: &Vec<Patch>) -> bool {
    for patch in patches {
        let result = apply_patch(patch);
        if !result {
            return false;
        }
    }
    true
}

fn apply_patch(patch: &Patch) -> bool {
    let status = Command::new("patch")
                         .arg("-i")
                         .arg(&patch.path)
                         .status()
                         .expect("Failed to patch");
    if !status.success() {
        return false;
    };
    let status = Command::new("git")
                         .arg("add")
                         .arg("--all")
                         .status()
                         .expect("Unable to git add");
    if !status.success() {
        return false;
    };
    let status = Command::new("git")
                         .arg("commit")
                         .arg("-m")
                         .arg(&patch.message)
                         .status()
                         .expect("Unable to git commit");
    if !status.success() {
        return false;
    };
    let status = Command::new("git")
                         .arg("format-patch")
                         .arg("-1")
                         .status()
                         .expect("Unable to git format-patch");
    if !status.success() {
        return false;
    };
    true
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

