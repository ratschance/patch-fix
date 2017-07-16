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
        (@arg start_num: -n --start-number +takes_value "Start number for the patches")
        (@arg patch: +required "Patch or directory with patches to apply")
    ).get_matches();

    let mut patch_file = PathBuf::from(matches.value_of("patch").unwrap());
    let signed = matches.value_of("signed");
    let start_num = matches.value_of("start_num").unwrap_or("1");
    let start_num = start_num.parse::<usize>().unwrap();


    if patch_file.is_dir() {
        patch_file.push("*.patch");
        match enumerate_patches(&patch_file, &signed) {
            Some(patches) => {
                apply_patches(&patches, start_num);
            }
            None => {
                println!("Unable to continue");
                return; 
            }
        }
    }
}

fn apply_patches(patches: &Vec<Patch>, start_num: usize) -> bool {
    for (num, patch) in patches.iter().enumerate() {
        let result = apply_patch(patch, start_num + num);
        if !result {
            return false;
        }
    }
    true
}

fn apply_patch(patch: &Patch, patch_num: usize) -> bool {
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
                         .arg("--start-number")
                         .arg(patch_num.to_string())
                         .status()
                         .expect("Unable to git format-patch");
    if !status.success() {
        return false;
    };
    true
}

fn enumerate_patches(dir: &PathBuf, signature: &Option<&str>) -> Option<Vec<Patch>> {
    let mut patches: Vec<Patch> = Vec::new();
    for entry in glob(dir.to_str().unwrap()).expect("Failed to read glob") {
        match entry {
            Ok(path_buf) => {
                match parse_patch(path_buf.as_path(), signature) {
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

