mod field_finder;
mod converter;

extern crate serde_json;
extern crate toml_edit;

use std::{fs, io, io::prelude::*, io::Error, io::ErrorKind};

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value as JValue};
use toml_edit::{array, table, value, Array, Document, Item, Value};

use crate::field_finder::get_field;
use crate::converter::json_serde_to_toml;

/**
 *  we have two operations we can do on the toml file:
 *  1. put field - creates the field if it doesn't already exist and sets it
 *  2. remove field - removes the field if it exists
 */

#[derive(Serialize, Deserialize)]
struct Op {
    Op: String,
    Field: String,
    Value: Option<String>,
}

// Reads from stdin a json that describes what operation to
// perform on the toml file. Returns either "success" or
// a message that starts with "error".
fn main() {
    let dotreplit_filepath = ".replit";

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
                let dotreplit_contents = match fs::read_to_string(dotreplit_filepath) {
                    Ok(contents) => contents,
                    Err(_) => {
                        println!("error: could not read file");
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
                for op in json {
                    let op_res = match op.Op.as_str() {
                        "put" => handle_put(op.Field, op.Value, &mut doc),
                        "remove" => handle_remove(op.Field, &mut doc),
                        _ => Err(Error::new(ErrorKind::Other, "Unexpected op type")),
                    };

                    if op_res.is_err() {
                        println!("error: {}", op_res.unwrap_err());
                        error_encountered = true;
                    }
                }

                if error_encountered {
                    println!("error: could not perform some dotreplit op");
                    continue;
                } 

                // write the file back to disk
                match fs::write(dotreplit_filepath, doc.to_string()) {
                    Ok(_) => println!("success"),
                    Err(_) => println!("error: could not write to file"),
                }
            },
            Err(_) => {
                println!("error: could not read line");
            }
        }
    }
}

fn handle_put(field: String, value_opt: Option<String>, doc: &mut Document) -> Result<(), Error> {
    let mut fields = field.split('/').collect();

    if fields.len() < 1 {
        return Err(Error::new(ErrorKind::Other, "Field path is empty"));
    }

    let final_field = match get_field(fields, doc) {
        Ok(final_field) => final_field,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Could not find field")),
    };

    if value_opt.is_none() {
        return Err(Error::new(
                ErrorKind::Other,
                "Expected value to be none null"
        ));
    }
    let value = value_opt.unwrap();

    let field_value = value.as_str();

    let json_field_value = match from_str(&field_value) {
        Ok(json_field_value) => json_field_value,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: value field in put request is not json"
            ));
        }
    };

    let toml = match json_serde_to_toml(&json_field_value) {
        Ok(converted_toml) => converted_toml,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    };

    final_field = toml;

    // doc[field_name] = converted_toml;

    return Ok(());
}

fn handle_remove(field: String, doc: &mut Document) -> Result<(), Error> {
    let mut fields: Vec<&str> = field.split('/').collect();

    if fields.len() < 1 {
        return Err(Error::new(ErrorKind::Other, "Field path is empty"));
    }

    let last_field = fields.pop().unwrap();

    if fields.len() == 0 {
        doc.remove(last_field);

        return Ok(());
    } 

    let field = match get_field(fields, doc) {
        Ok(field) => field,
        Err(e) => return Err(e),
    };

    if field.is_array() {
        let field_array = field.as_array_mut().unwrap();

        let array_index = match last_field.parse::<usize>() {
            Ok(index) => index,
            Err(_) => return Err(Error::new(ErrorKind::Other, "could not parse array index")),
        };

        if array_index >= field_array.len() {
            return Err(Error::new(
                ErrorKind::Other,
                "error: array index out of bounds",
            ));
        }

        field_array.remove(array_index);
    } else if field.is_table() {
        let field_table = field.as_table_mut().unwrap();

        field_table.remove(last_field);
    } else {
        return Err(Error::new(
            ErrorKind::Other,
            "error: field is not an array or table",
        ));
    }


    return Ok(());
}

