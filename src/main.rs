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
        (@arg strip: -p --strip +takes_value "Number of leading slashes to remove. See -p from patch for more info")
        (@arg patch: +required "Patch or directory with patches to apply")
    ).get_matches();

    let mut patch_file = PathBuf::from(matches.value_of("patch").unwrap());
    let signed = matches.value_of("signed");
    let start_num = matches.value_of("start_num").unwrap_or("1");
    let start_num = start_num.parse::<usize>().unwrap();
    let strip_num = matches.value_of("strip").unwrap_or("0");


    if patch_file.is_dir() {
        patch_file.push("*.patch");
        match enumerate_patches(&patch_file, &signed) {
            Some(patches) => {
                apply_patches(&patches, start_num, strip_num);
            }
            None => {
                println!("Unable to continue");
                return; 
            }
        }
    }
}

fn apply_patches(patches: &Vec<Patch>, start_num: usize, strip_num: &str) -> bool {
    for (num, patch) in patches.iter().enumerate() {
        let result = apply_patch(patch, start_num + num, strip_num);
        if !result {
            return false;
        }
    }
    true
}

fn apply_patch(patch: &Patch, patch_num: usize, strip_num: &str) -> bool {
    let status = Command::new("patch")
                         .arg("-i")
                         .arg(&patch.path)
                         .arg(format!("-p{}", strip_num))
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

