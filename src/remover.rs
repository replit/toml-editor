use std::{io::Error, io::ErrorKind};
use crate::field_finder::{get_field, TomlValue};
use toml_edit::{Document, Array, ArrayOfTables, InlineTable, Table};

pub fn handle_remove(field: String, doc: &mut Document) -> Result<(), Error> {
    let mut path_split = field.split('/').map(|s| s.to_string()).collect::<Vec<String>>();

    let last_field = match path_split.pop() {
        Some(last_field) => last_field.to_string(),
        None => return Err(Error::new(ErrorKind::Other, "Path is empty")),
    };

    let field = match get_field(&path_split, &last_field.to_string(), doc) {
        Ok(field) => field,
        Err(e) => return Err(e),
    };

    match field {
        TomlValue::Table(table) => remove_in_table(table, last_field),
        TomlValue::Array(array) => remove_in_array(array, last_field),
        TomlValue::ArrayOfTables(array) => remove_in_array_of_tables(array, last_field),
        TomlValue::InlineTable(table) => remove_in_inline_table(table, last_field),
        TomlValue::Value(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: cannot remove_in non array/table value",
            ));
        },
    }
}

fn remove_in_array(array: &mut Array, last_field: String) -> Result<(), Error> {
    let array_index = match last_field.parse::<usize>() {
        Ok(array_index) => array_index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse array index",
            ));
        }
    };

    array.remove(array_index);
    Ok(())
}

fn remove_in_array_of_tables(array: &mut ArrayOfTables, last_field: String) -> Result<(), Error> {
    let array_index = match last_field.parse::<usize>() {
        Ok(array_index) => array_index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse array index",
            ));
        }
    };

    array.remove(array_index);
    Ok(())
}

fn remove_in_table(table: &mut Table, last_field: String) -> Result<(), Error> {
    table.remove(last_field.as_str());
    Ok(())
}

fn remove_in_inline_table(inline_table: &mut InlineTable, last_field: String) -> Result<(), Error> {
    inline_table.remove(last_field.as_str());
    Ok(())
}
