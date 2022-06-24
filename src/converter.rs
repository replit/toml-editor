use std::{io::Error, io::ErrorKind};

use serde_json::Value as JValue;
use toml_edit::{array, table, value, Array, Item, Value, InlineTable};

// converts json objects to toml objects
pub fn json_to_toml(json: &JValue, inline: bool) -> Result<Item, Error> {
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
                .map(|v| json_to_toml(v, inline))
                .collect::<Result<Vec<Item>, Error>>();
            match items {
                Ok(items) => create_toml_array(items, inline),
                Err(e) => Err(e),
            }
        }
        JValue::Object(o) => {
            let items = o
                .iter()
                .map(|(k, v)| Ok((k.clone(), json_to_toml(v, inline)?)))
                .collect::<Result<Vec<(String, Item)>, Error>>();
            match items {
                Ok(items) => create_toml_table(items, inline),
                Err(e) => Err(e),
            }
        }
    }
}

fn create_toml_table(items: Vec<(String, Item)>, inline: bool) -> Result<Item, Error> {
    let mut output_table = if inline { table() } else { Item::Value(Value::InlineTable(InlineTable::new())) };

    let table_like_item = match output_table.as_table_like_mut() {
        Some(t) => t,
        None => return Err(Error::new(ErrorKind::Other, "error: could not create table like")),
    };

    for (item_key, item_value) in items {
        table_like_item.insert(item_key.as_str(), item_value);
    }

    return Ok(output_table);
}

fn create_toml_array(items: Vec<Item>, inline: bool) -> Result<Item, Error> {
    if inline { create_toml_inline_array(items) } else { create_toml_array_of_tables(items) }
}

fn create_toml_inline_array(items: Vec<Item>) -> Result<Item, Error> {
    let mut output_array = Array::new();
    for item in items {
        match item {
            Item::Value(v) => output_array.push(v),
            _ => return Err(Error::new(ErrorKind::Other, "error: could not create inline array")),
        }
    }

    return Ok(value(output_array));
}

fn create_toml_array_of_tables(items: Vec<Item>) -> Result<Item, Error> {
    let mut output_array = array();
    let output_array_tables = match output_array.as_array_of_tables_mut() {
        Some(t) => t,
        None => return Err(Error::new(ErrorKind::Other, "error: could not create array of tables")),
    };

    for item in items {
        let table = match item.into_table() {
            Ok(t) => t,
            Err(_) => return Err(Error::new(ErrorKind::Other, "error: could not convert item to table in array of tables")),
        };

        output_array_tables.push(table);
    }

    return Ok(output_array);
}
