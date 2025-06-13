mod adder;
mod converter;
mod field_finder;
mod remover;
mod traversal;

use std::fs;
use std::path::{Path, PathBuf};
use std::{io, io::prelude::*};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use toml_edit::DocumentMut;

use crate::adder::handle_add;
use crate::remover::handle_remove;
use crate::traversal::TraverseOps;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = ".replit")]
    path: PathBuf,

    #[clap(short, long, value_parser, default_value = "false")]
    // Whether or not to write this value directly to the file,
    // or just print it as part of the return message.
    return_output: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "op")]
enum OpKind {
    /// Creates the field if it doesn't already exist and sets it
    #[serde(rename = "add")]
    Add(AddOp),

    /// Gets the value at the specified path, returned as JSON
    #[serde(rename = "get")]
    Get { path: String },

    /// Removes the field if it exists
    #[serde(rename = "remove")]
    Remove { path: String },
}

#[derive(Serialize, Deserialize)]
struct AddOp {
    path: Option<String>,
    table_header_path: Option<String>,
    dotted_path: Option<String>,
    value: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Res {
    status: String,
    message: Option<String>,
    results: Vec<Value>,
}

// Reads from stdin a json that describes what operation to
// perform on the toml file. Returns either "success" or
// a message that starts with "error".
fn main() -> Result<()> {
    let args = Args::parse();
    let dotreplit_filepath = args.path;

    // read line by line from stdin until eof
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let res = handle_message(&dotreplit_filepath, &line, args.return_output);

        let res_json = serde_json::to_string(&res)?;
        println!("{}", res_json);
    }

    Ok(())
}

fn handle_message(dotreplit_filepath: &Path, msg: &str, return_output: bool) -> Res {
    match do_edits(dotreplit_filepath, msg, return_output) {
        Ok((doc, outs)) => Res {
            status: "success".to_string(),
            message: if return_output { Some(doc) } else { None },
            results: outs,
        },
        Err(err) => Res {
            status: "error".to_string(),
            message: Some(err.to_string()),
            results: vec![],
        },
    }
}

fn do_edits(
    dotreplit_filepath: &Path,
    msg: &str,
    return_output: bool,
) -> Result<(String, Vec<Value>)> {
    // parse line as json
    let json: Vec<OpKind> = from_str(msg)?;

    // we need to re-read the file each time since the user might manually edit the
    // file and so we need to make sure we have the most up to date version.
    let dotreplit_contents = match fs::read_to_string(dotreplit_filepath) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => "".to_string(), // if .replit doesn't exist start with an empty one
        Err(_) => return Err(anyhow!("error: reading file - {:?}", &dotreplit_filepath)),
    };

    let mut doc = dotreplit_contents
        .parse::<DocumentMut>()
        .with_context(|| format!("error: parsing file - {:?}", &dotreplit_filepath))?;

    let mut outputs: Vec<Value> = vec![];
    for op in json {
        match op {
            OpKind::Add(op) => {
                handle_add(&mut doc, op)?;
                outputs.push(json!("ok"));
            }
            OpKind::Get { path } => match traversal::traverse(TraverseOps::Get, &mut doc, &path) {
                Ok(value) => outputs.push(value.unwrap_or_default()),
                Err(error) => {
                    eprintln!("Error processing {}: {}", path, error);
                    outputs.push(Value::Null)
                }
            },
            OpKind::Remove { path } => {
                handle_remove(&path, &mut doc)?;
                outputs.push(json!("ok"));
            }
        }
    }

    if return_output {
        return Ok((doc.to_string(), outputs));
    }

    // write the file back to disk
    let new_contents = doc.to_string();
    if dotreplit_contents != new_contents {
        fs::write(dotreplit_filepath, new_contents)
            .with_context(|| format!("error: writing file: {:?}", &dotreplit_filepath))?;
    }
    Ok(("".to_string(), outputs))
}
