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
use toml_edit::{table, value, DocumentMut, Table, Item};
use crate::field_finder::{get_field, DoInsert, TomlValue};

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
enum OpKind {
    /// Creates the field if it doesn't already exist and sets it
    #[serde(rename = "add")]
    Add,

    /// Gets the value at the specified path, returned as JSON
    #[serde(rename = "get")]
    Get,

    /// Removes the field if it exists
    #[serde(rename = "remove")]
    Remove,
}

#[derive(Serialize, Deserialize)]
struct Op {
    op: OpKind,
    path: String,
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

    // return play();

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

// fn play() -> Result<()> {
//     let toml = fs::read_to_string("test.toml")
//     .expect("Should have been able to read the file");
//     let mut doc = toml.parse::<DocumentMut>().expect("invalid doc");
//     // let mut module_config = Table::new();
//     // module_config.set_dotted(true);
//     // let mut bundles = Table::new();
//     // bundles.set_dotted(true);
//     let table_header_path = vec!["moduleConfig"].iter().map(|s| s.to_string()).collect::<Vec<String>>();
//     let dotted_path = vec!["interpreters", "ruby", "enable"].iter().map(|s| s.to_string()).collect::<Vec<String>>();
//     _ = table_header_adder::add_value_with_table_header_and_dotted_path(&mut doc, &table_header_path, &dotted_path, value(false));
//     // _ = add_value_with_dotted_path(&mut bundles, &path, value(true));
//     // module_config.insert("bundles", Item::Table(bundles));
//     // let mut go = Table::new();
//     // go.insert("enable", value(true));
//     // go.set_dotted(true);
//     // bundles.insert("go", toml_edit::Item::Table(go));
//     // module_config.insert("bundles", toml_edit::Item::Table(bundles));


//     // doc.insert("moduleConfig", Item::Table(module_config));
//     // println!("length: {}", path.len());
//     // return insert_value_with_dotted_path(&mut go, &path, value(true));
//     // match doc.get_mut("moduleConfig") {
//     //     None | Some(toml_edit::Item::None) =>
//     //     {
//     //         let mut table = Table::new();
//     //         insert_dotted_path(table, dotted_path, value);
//     //         doc.insert("moduleConfig", table);
//     //     }
//     //     Some(toml_edit::Item::Table(existing_table)) => {
//     //         insert_dotted_path(existing_table, dotted_path, value);
//     //     }
//     //     ...
//     // };


//     // let doc_table = doc.as_table_mut();
//     // let module_config = doc_table.get_mut("moduleConfig");
//     // println!("module_config {:?}", module_config);
//     // let table = match module_config {
//     //     None => {
//     //         doc["moduleConfig"] = table();
//     //     }
//     //     Some( => ()
//     // }


//     // println!("module_config: {}", module_config);
//     println!("{}", doc.to_string());
//     Ok(())
// }

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
    let json: Vec<Op> = from_str(msg)?;

    // we need to re-read the file each time since the user might manually edit the
    // file and so we need to make sure we have the most up to date version.
    let dotreplit_contents = match fs::read_to_string(&dotreplit_filepath) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => "".to_string(), // if .replit doesn't exist start with an empty one
        Err(_) => return Err(anyhow!("error: reading file - {:?}", &dotreplit_filepath)),
    };

    let mut doc = dotreplit_contents
        .parse::<DocumentMut>()
        .with_context(|| format!("error: parsing file - {:?}", &dotreplit_filepath))?;

    let mut changed: bool = false;
    let mut outputs: Vec<Value> = vec![];
    for op in json {
        let path = op.path;
        match op.op {
            OpKind::Add => {
                let value = op.value.context("error: expected value to add")?;
                changed = true;
                handle_add(
                    &path,
                    op.table_header_path,
                    op.dotted_path,
                    &value,
                    &mut doc
                )?;
                outputs.push(json!("ok"));
            }
            OpKind::Get => match traversal::traverse(TraverseOps::Get, &mut doc, &path) {
                Ok(value) => outputs.push(value.unwrap_or_default()),
                Err(error) => {
                    eprintln!("Error processing {}: {}", path, error);
                    outputs.push(Value::Null)
                }
            },
            OpKind::Remove => {
                changed = true;
                handle_remove(&path, &mut doc)?;
                outputs.push(json!("ok"));
            }
        }
    }

    if return_output {
        return Ok((doc.to_string(), outputs));
    }

    // write the file back to disk
    if changed {
        fs::write(&dotreplit_filepath, doc.to_string())
            .with_context(|| format!("error: writing file: {:?}", &dotreplit_filepath))?;
    }
    Ok(("".to_string(), outputs))
}
