mod adder;
mod converter;
mod field_finder;
mod remover;

extern crate serde_json;
extern crate toml_edit;

use std::fs;
use std::{io, io::prelude::*, io::Error, io::ErrorKind};

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use toml_edit::Document;

use crate::adder::handle_add;
use crate::remover::handle_remove;

use clap::Parser;

/**
 *  we have two operations we can do on the toml file:
 *  1. add field - creates the field if it doesn't already exist and sets it
 *  2. remove field - removes the field if it exists
 */

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = ".replit")]
    path: String,
}

#[derive(Serialize, Deserialize)]
struct Op {
    op: String,
    path: String,
    value: Option<String>,
}

// Reads from stdin a json that describes what operation to
// perform on the toml file. Returns either "success" or
// a message that starts with "error".
fn main() {
    let args = Args::parse();
    let dotreplit_filepath = args.path;

    // read line by line from stdin until eof
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                // parse line as json
                let json: Vec<Op> = match from_str(&line) {
                    Ok(json_val) => json_val,
                    Err(_) => {
                        println!("error: could not parse json ");
                        continue;
                    }
                };

                // we need to re-read the file each time since the user might manually edit the
                // file and so we need to make sure we have the most up to date version.
                let dotreplit_contents = match fs::read_to_string(&dotreplit_filepath) {
                    Ok(contents) => contents,
                    Err(_) => {
                        println!("error: could not read file {}", dotreplit_filepath);
                        continue;
                    }
                };

                let mut doc = match dotreplit_contents.parse::<Document>() {
                    Ok(doc_contents) => doc_contents,
                    Err(_) => {
                        println!("error: could not parse toml");
                        continue;
                    }
                };

                let mut error_encountered: bool = false;
                let mut errors: Vec<String> = Vec::new();
                for op in json {
                    let op_res = match op.op.as_str() {
                        "add" => handle_add(op.path, op.value, &mut doc),
                        "remove" => handle_remove(op.path, &mut doc),
                        _ => Err(Error::new(ErrorKind::Other, "Unexpected op type")),
                    };

                    if let Err(err) = op_res {
                        error_encountered = true;
                        errors.push(err.to_string());
                    }
                }

                if error_encountered {
                    println!(
                        "error: could not perform some dotreplit op - {}",
                        errors.join("|")
                    );
                    continue;
                }

                // write the file back to disk
                match fs::write(&dotreplit_filepath, doc.to_string()) {
                    Ok(_) => println!("success"),
                    Err(_) => println!("error: could not write to file"),
                }
            }
            Err(_) => {
                println!("error: could not read line");
            }
        }
    }
}
