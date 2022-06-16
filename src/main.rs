extern crate serde_json;
extern crate toml_edit;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value as JValue};
use std::{fs, io, io::prelude::*, io::Error, io::ErrorKind};
use toml_edit::{array, table, value, Array, Document, Item, Value};

/**
 *  we have two operations we can do on the toml file:
 *  1. put field - creates the field if it doesn't already exist and sets it
 *  2. remove field - removes the field if it exists
 */

#[derive(Serialize, Deserialize)]
enum Command {
    #[allow(non_camel_case_types)]
    put(Put),
    #[allow(non_camel_case_types)]
    remove(Remove),
}

#[derive(Serialize, Deserialize)]
struct Put {
    field: String,
    value: JValue,
}

#[derive(Serialize, Deserialize)]
struct Remove {
    field: String,
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
                let json_res = from_str(&line);
                if json_res.is_err() {
                    println!("error: could not parse json, {}", line);
                    continue;
                }
                let json: Command = json_res.unwrap();

                // we need to re-read the file each time since the user might manually edit the
                // file and so we need to make sure we have the most up to date version.
                let dotreplit_contents_res = fs::read_to_string(dotreplit_filepath);
                if dotreplit_contents_res.is_err() {
                    println!("error: could not read file, {}", dotreplit_filepath);
                    continue;
                }
                let dotreplit_contents = dotreplit_contents_res.unwrap();

                let doc_res = dotreplit_contents.parse::<Document>();
                if doc_res.is_err() {
                    println!("error: could not parse .replit, {}", dotreplit_contents);
                    continue;
                }
                let mut doc = doc_res.unwrap();

                let op_res = match json {
                    Command::put(put) => handle_put(put, &mut doc),
                    Command::remove(remove) => handle_remove(remove, &mut doc),
                };

                if op_res.is_err() {
                    println!("error: {}", op_res.unwrap_err());
                    continue;
                }

                println!("success");

                // write the file back to disk
                let write_res = fs::write(dotreplit_filepath, doc.to_string());
                if write_res.is_err() {
                    println!("error: could not write to file, {}", dotreplit_filepath);
                    continue;
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_put(put_obj: Put, doc: &mut Document) -> Result<(), Error> {
    let field_name = put_obj.field.as_str();
    let field_value = put_obj.value;

    let converted_toml_res = json_serde_to_toml(&field_value);
    if converted_toml_res.is_err() {
        return Err(Error::new(
            ErrorKind::Other,
            "error: could not convert json to toml",
        ));
    }
    let converted_toml = converted_toml_res.unwrap();

    doc[field_name] = converted_toml;

    return Ok(());
}

fn handle_remove(remove_obj: Remove, doc: &mut Document) -> Result<(), Error> {
    let field_name = remove_obj.field.as_str();
    doc.remove(field_name);

    return Ok(());
}

fn create_toml_array(items: Vec<Item>) -> Result<Item, Error> {
    // toml_edit treats arrays of values and arrays of tables differently
    // and so we need to check if it's an array of tables or array of values
    // and handle them accordingly.
    if items.len() == 0 {
        return Ok(array());
    }

    if items[0].is_table() {
        let mut output_array = array();
        let output_array_table_res = output_array.as_array_of_tables_mut();
        if output_array_table_res.is_none() {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not create array",
            ));
        }
        let output_array_table = output_array_table_res.unwrap();

        for item in items {
            let table_res = item.into_table();
            if table_res.is_err() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "error: could not convert item to table",
                ));
            }
            let table = table_res.unwrap();

            output_array_table.push(table);
        }

        return Ok(output_array);
    } else {
        let mut output_array = Array::new();
        for item in items {
            let value_res = item.into_value();
            if value_res.is_err() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "error: could not convert item to value",
                ));
            }
            let value = value_res.unwrap();
            match value {
                Value::String(s) => output_array.push(s.into_value()),
                Value::Integer(i) => output_array.push(i.into_value()),
                Value::Float(f) => output_array.push(f.into_value()),
                Value::Boolean(b) => output_array.push(b.into_value()),
                Value::Datetime(dt) => output_array.push(dt.into_value()),
                Value::Array(a) => output_array.push(a),
                Value::InlineTable(it) => output_array.push(it),
            }
        }

        return Ok(value(output_array));
    }
}

fn create_toml_table(items: Vec<(String, Item)>) -> Item {
    let mut output_table = table();

    for (item_key, item_value) in items {
        output_table[item_key.as_str()] = item_value;
    }

    return output_table;
}

// converts json objects to toml objects
fn json_serde_to_toml(json: &JValue) -> Result<Item, Error> {
    match json {
        JValue::Null => Ok(Item::None),
        JValue::Bool(b) => Ok(value(*b)),
        JValue::Number(n) => match n.as_f64() {
            Some(f) => Ok(value(f)),
            None => return Err(Error::new(ErrorKind::Other, "unsupported number type")),
        },
        JValue::String(s) => Ok(value(s.clone())),
        JValue::Array(a) => {
            let items = a
                .iter()
                .map(|v| json_serde_to_toml(v))
                .collect::<Result<Vec<Item>, Error>>();
            match items {
                Ok(items) => create_toml_array(items),
                Err(e) => Err(e),
            }
        }
        JValue::Object(o) => {
            let items = o
                .iter()
                .map(|(k, v)| Ok((k.clone(), json_serde_to_toml(v)?)))
                .collect::<Result<Vec<(String, Item)>, Error>>();
            match items {
                Ok(items) => Ok(create_toml_table(items)),
                Err(e) => Err(e),
            }
        }
    }
}
