use std::{io::Error, io::ErrorKind};
use serde_json::{from_str, Value as JValue};
use crate::converter::json_to_toml;
use crate::field_finder::{get_field, TomlValue};
use toml_edit::{Item, Document, Array, ArrayOfTables, Value, InlineTable, Table};

pub fn handle_add(field: String, value_opt: Option<String>, doc: &mut Document) -> Result<(), Error> {
    let mut path_split = field.split('/').map(|s| s.to_string()).collect::<Vec<String>>();

    let last_field = match path_split.pop() {
        Some(last_field) => last_field.to_string(),
        None => return Err(Error::new(ErrorKind::Other, "Path is empty")),
    };

    let final_field_value = match get_field(&path_split, &last_field, doc) {
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

    let field_value_json = match from_str(&value.as_str()) {
        Ok(json_field_value) => json_field_value,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: value field in add request is not json"
            ));
        }
    };

    match final_field_value {
        TomlValue::Table(table) => add_in_table(table, last_field, &field_value_json),
        TomlValue::ArrayOfTables(array) => add_in_array_of_tables(array, last_field, &field_value_json),
        TomlValue::Array(array) => add_in_array(array, last_field, &field_value_json),
        TomlValue::InlineTable(table) => add_in_inline_table(table, last_field, &field_value_json),
        TomlValue::Value(value) => add_in_generic_value(value, last_field, &field_value_json),
    }
}

fn add_in_table(table: &mut Table, last_field: String, value: &JValue) -> Result<(), Error> {
    let toml = match json_to_toml(value, false) {
        Ok(toml) => toml,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    };

    table.insert(last_field.as_str(), toml);
    Ok(())
}

fn add_in_array_of_tables(array: &mut ArrayOfTables, last_field: String, value: &JValue) -> Result<(), Error> {
    let toml = match json_to_toml(value, false) {
        Ok(toml) => toml,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    };

    let insert_at_index = match last_field.parse::<usize>() {
        Ok(index) => index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse last_field as usize",
            ));
        }
    };

    match toml {
        Item::Table(table) => {
            if insert_at_index >= array.len() {
                array.push(table);
            } else {
                let table_to_modify = match array.get_mut(insert_at_index) {
                    Some(table) => table,
                    None => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "error: could not get mutable reference to table at index",
                        ));
                    }
                };

                *table_to_modify = table;
            }
        },
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    }

    Ok(())
}

fn add_in_inline_table(table: &mut InlineTable, last_field: String, value: &JValue) -> Result<(), Error> {
    let toml = match json_to_toml(value, true) {
        Ok(toml) => toml,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    };

    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            match table.insert(last_field.as_str(), value) {
                Some(_) => {},
                None => return Err(Error::new(
                    ErrorKind::Other,
                    "error: could not insert value into inline table",
                )),
            };
        },
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to inline toml",
            ));
        }
    }

    Ok(())
}

fn add_in_array(array: &mut Array, last_field: String, value: &JValue) -> Result<(), Error> {
    let toml = match json_to_toml(value, true) {
        Ok(toml) => toml,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    };

    let insert_at_index = match last_field.parse::<usize>() {
        Ok(index) => index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse last_field as usize",
            ));
        }
    };

    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            if insert_at_index >= array.len() {
                array.push(value);
            } else {
                let value_to_modify = match array.get_mut(insert_at_index) {
                    Some(value) => value,
                    None => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "error: could not get mutable reference to value at index",
                        ));
                    }
                };

                *value_to_modify = value;
            }
        },
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    }

    Ok(())
}

fn add_in_generic_value(generic_value: &mut Value, last_field: String, value: &JValue) -> Result<(), Error> {
    match generic_value {
        Value::InlineTable(table) => add_in_inline_table(table, last_field, value),
        Value::Array(array) => add_in_array(array, last_field, value),
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not add into generic value",
            ));
        }
    }
}
