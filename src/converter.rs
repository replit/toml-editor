use std::{io::Error, io::ErrorKind};

use serde_json::Value as JValue;
use toml_edit::{array, table, value, Array, Item, Value};

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
pub fn json_serde_to_toml(json: &JValue) -> Result<Item, Error> {
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
