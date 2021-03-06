mod adder;
mod converter;
mod field_finder;
mod remover;

use std::fs;
use std::path::{Path, PathBuf};
use std::{io, io::prelude::*};

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use toml_edit::Document;

use crate::adder::handle_add;
use crate::remover::handle_remove;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value = ".replit")]
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
enum OpKind {
    /// Creates the field if it doesn't already exist and sets it
    #[serde(rename = "add")]
    Add,

    /// Removes the field if it exists
    #[serde(rename = "remove")]
    Remove,
}

#[derive(Serialize, Deserialize)]
struct Op {
    op: OpKind,
    path: String,
    value: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Res {
    status: String,
    message: Option<String>,
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
        let res = match handle_message(&dotreplit_filepath, &line) {
            Ok(_) => Res {
                status: "success".to_string(),
                message: None,
            },
            Err(err) => Res {
                status: "error".to_string(),
                message: Some(err.to_string()),
            },
        };

        let res_json = serde_json::to_string(&res)?;
        println!("{}", res_json);
    }

    Ok(())
}

fn handle_message(dotreplit_filepath: &Path, msg: &str) -> Result<()> {
    // parse line as json
    let json: Vec<Op> = from_str(msg)?;

    // we need to re-read the file each time since the user might manually edit the
    // file and so we need to make sure we have the most up to date version.
    let dotreplit_contents = fs::read_to_string(&dotreplit_filepath)
        .with_context(|| format!("error: reading file - {:?}", &dotreplit_filepath))?;

    let mut doc = dotreplit_contents
        .parse::<Document>()
        .with_context(|| format!("error: parsing file - {:?}", &dotreplit_filepath))?;

    for op in json {
        match op.op {
            OpKind::Add => {
                let value = op.value.context("error: expected value to add")?;
                handle_add(&op.path, &value, &mut doc)?
            }
            OpKind::Remove => handle_remove(&op.path, &mut doc)?,
        }
    }

    // write the file back to disk
    fs::write(&dotreplit_filepath, doc.to_string())
        .with_context(|| format!("error: writing file: {:?}", &dotreplit_filepath))?;
    Ok(())
}
