use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::io::prelude::*;

const R_HASH_LINE: &'static str = r"^From ([0-9a-z]{40}) [A-Z][a-z]{2} ([A-Z][a-z]{2} \d+ \d{2}:\d{2}:\d{2} \d{4})$";
const R_AUTHOR_LINE: &'static str = r"^From: (.+)$";
const R_DATE_LINE: &'static str = r"^Date: [A-Z][a-z]{2}, (\d+ [A-Z][a-z]{2} \d{4} \d{2}:\d{2}:\d{2} -\d{4})$";
const R_SUBJECT_LINE: &'static str = r"^Subject: \[PATCH\s?\d*/*\d*\] (.+)$";

enum ParseStates {
    Init,
    Author,
    Date,
    Subject,
    Message,
    Finish,
    Invalid,
}

#[derive(Default)]
pub struct Patch {
    hash: String,
    from_date: String,
    orig_author: String,
    orig_date: String,
    pub message: String,
    pub path: String,
}

pub fn parse_patch(path: &Path, signature: &Option<&str>) -> Option<Patch> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(&file);
    let mut current_state = ParseStates::Init;
    let mut patch = Patch { path: path.to_str().unwrap().to_owned(), ..Default::default() };

    for line in reader.lines() {
        let line = line.unwrap();
        match current_state {
            ParseStates::Init => {
                lazy_static! {
                    static ref RE_INIT: Regex = Regex::new(R_HASH_LINE).unwrap();
                }
                match RE_INIT.captures(&line) {
                    Some(captures) => {
                        patch.hash.push_str(captures.get(1).unwrap().as_str());
                        patch.from_date.push_str(captures.get(2).unwrap().as_str());
                        current_state = ParseStates::Author;
                    }
                    None => {
                        println!("Invalid hash line.");
                        current_state = ParseStates::Invalid;
                    }
                }
            }
            ParseStates::Author => {
                lazy_static! {
                    static ref RE_AUTHOR: Regex = Regex::new(R_AUTHOR_LINE).unwrap();
                }
                match RE_AUTHOR.captures(&line) {
                    Some(captures) => {
                        patch.orig_author.push_str(captures.get(1).unwrap().as_str());
                        current_state = ParseStates::Date;
                    }
                    None => {
                        println!("Invalid author line.");
                        current_state = ParseStates::Invalid;
                    }
                }
            }
            ParseStates::Date => {
                lazy_static! {
                    static ref RE_DATE: Regex = Regex::new(R_DATE_LINE).unwrap();
                }
                match RE_DATE.captures(&line) {
                    Some(captures) => {
                        patch.orig_date.push_str(captures.get(1).unwrap().as_str());
                        current_state = ParseStates::Subject;
                    }
                    None => {
                        println!("Invalid date line.");
                        current_state = ParseStates::Invalid;
                    }
                }
            }
            ParseStates::Subject => {
                lazy_static! {
                    static ref RE_SUB: Regex = Regex::new(R_SUBJECT_LINE).unwrap();
                }
                match RE_SUB.captures(&line) {
                    Some(captures) => {
                        patch.message.push_str(captures.get(1).unwrap().as_str());
                        patch.message.push_str("\n");
                        current_state = ParseStates::Message;
                    }
                    None => {
                        println!("Invalid subject line.");
                        current_state = ParseStates::Invalid;
                    }
                }
            }
            ParseStates::Message => {
                if &line == "---" {
                    if let Some(sig) = *signature {
                            patch.message.push_str("Signed-off-by: ");
                            patch.message.push_str(sig);
                            patch.message.push_str("\n");
                    }
                    current_state = ParseStates::Finish;
                } else {
                    patch.message.push_str(&line);
                    patch.message.push_str("\n");
                }
            }
            ParseStates::Finish => {
                return Some(patch)
            }
            ParseStates::Invalid => {
                println!("Failed to parse: {}", patch.path);
                break;
            }
        }
    }
    None
}

