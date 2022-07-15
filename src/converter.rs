use anyhow::{bail, Context, Result};
use serde_json::Value as JValue;
use toml_edit::{value, Array, ArrayOfTables, InlineTable, Item, Table, Value};

// converts json objects to toml objects
pub fn json_to_toml(json: &JValue, inline: bool) -> Result<Item> {
    match json {
        JValue::Null => Ok(Item::None),
        JValue::Bool(b) => Ok(value(*b)),
        JValue::Number(n) => match n.as_i64() {
            Some(i) => Ok(value(i)),
            None => Ok(value(
                n.as_f64()
                    .context("JSON number is not an integer or a float")?,
            )),
        },
        JValue::String(s) => Ok(value(s.clone())),
        JValue::Array(a) => {
            let items = a
                .iter()
                .map(|v| json_to_toml(v, inline))
                .collect::<Result<Vec<Item>, _>>()?;

            create_toml_array(items, inline)
        }
        JValue::Object(o) => {
            let items = o
                .iter()
                .map(|(k, v)| Result::<_>::Ok((k.clone(), json_to_toml(v, inline)?)))
                .collect::<Result<Vec<(String, Item)>, _>>()?;

            create_toml_table(items, inline)
        }
    }
}

fn create_toml_inline_table(items: Vec<(String, Item)>) -> Result<Item> {
    let mut output_table = InlineTable::new();

    for (k, v) in items {
        let item_value = match v {
            Item::Value(v) => v,
            _ => bail!("unsupported type"),
        };

        output_table.insert(k.as_str(), item_value);
    }

    Ok(Item::Value(Value::InlineTable(output_table)))
}

fn create_toml_block_table(items: Vec<(String, Item)>) -> Result<Item> {
    let mut output_table = Table::new();

    for (k, v) in items {
        output_table.insert(k.as_str(), v);
    }

    Ok(Item::Table(output_table))
}

fn create_toml_table(items: Vec<(String, Item)>, inline: bool) -> Result<Item> {
    if inline {
        create_toml_inline_table(items)
    } else {
        create_toml_block_table(items)
    }
}

fn every_item_is_table(items: &Vec<Item>) -> bool {
    for item in items {
        if let Item::Table(_) = item {
            continue;
        } else {
            return false;
        }
    }

    true
}

fn create_toml_array(items: Vec<Item>, inline: bool) -> Result<Item> {
    if !inline && every_item_is_table(&items) {
        create_toml_array_of_tables(items)
    } else {
        create_toml_inline_array(items)
    }
}

fn create_toml_inline_array(items: Vec<Item>) -> Result<Item> {
    let mut output_array = Array::new();
    for item in items {
        match item {
            Item::Value(v) => output_array.push(v),
            _ => bail!("could not create inline array"),
        }
    }

    Ok(value(output_array))
}

fn create_toml_array_of_tables(items: Vec<Item>) -> Result<Item> {
    let mut output_array = ArrayOfTables::new();

    for item in items {
        match item {
            Item::Table(t) => output_array.push(t),
            _ => bail!("could not create array of tables"),
        }
    }

    Ok(Item::ArrayOfTables(output_array))
}

#[cfg(test)]
mod converter_tests {
    use super::*;
    use serde_json::{from_str, Value as JValue};
    use toml_edit::Document;

    #[test]
    fn test_json_to_toml_array() {
        let json: JValue = from_str("[1, 2, 3]").unwrap();
        let toml = json_to_toml(&json, true).unwrap();
        let res = toml.to_string();
        assert_eq!(res, "[1, 2, 3]");
    }

    #[test]
    fn test_json_to_block_table() {
        let json: JValue = from_str("{\"a\": 1, \"b\": 2}").unwrap();
        let toml = json_to_toml(&json, false).unwrap();
        let res = toml.to_string();
        assert_eq!(res.trim(), "a = 1\nb = 2".trim());
    }

    #[test]
    fn test_json_to_inline_table() {
        let json: JValue = from_str("{\"a\": 1, \"b\": 2}").unwrap();
        let toml = json_to_toml(&json, true).unwrap();
        let res = toml.to_string();
        assert_eq!(res.trim(), "{ a = 1, b = 2 }".trim());
    }

    #[test]
    fn test_json_to_inline_table_with_array() {
        let json_string = r#"{
    "who": 123,
    "arr": [
        { "a": 1, "b": 2 },
        { "a": 3, "b": 4 }
    ]
}"#;
        let json: JValue = from_str(json_string).unwrap();
        let toml = json_to_toml(&json, true).unwrap();
        let res = toml.to_string();
        assert_eq!(
            res.trim(),
            "{ arr = [{ a = 1, b = 2 }, { a = 3, b = 4 }], who = 123 }".trim()
        );
    }

    #[test]
    fn test_json_to_toml_table_with_array_of_tables() {
        let json_string = r#"[
        { "a": 1, "b": 2 },
        { "a": 3, "b": 4 }
    ]"#;
        let json: JValue = from_str(json_string).unwrap();
        let toml_res = json_to_toml(&json, false);
        assert!(toml_res.is_ok());
        let mut doc = Document::new();
        doc["arr"] = toml_res.unwrap();

        let expected = r#"
[[arr]]
a = 1
b = 2

[[arr]]
a = 3
b = 4
"#;

        assert_eq!(doc.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_json_to_toml_table_with_deep_array() {
        let json_string = r#"[
        { "a": 1, "b": [1, 2, 3] },
        { "a": 3, "b": [2, 3, 4] }
    ]"#;
        let json: JValue = from_str(json_string).unwrap();
        let toml_res = json_to_toml(&json, false);
        assert!(toml_res.is_ok());
        let mut doc = Document::new();
        doc["arr"] = toml_res.unwrap();

        let expected = r#"
[[arr]]
a = 1
b = [1, 2, 3]

[[arr]]
a = 3
b = [2, 3, 4]
"#;

        assert_eq!(doc.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_json_to_toml_float() {
        let json: JValue = from_str("1.4").unwrap();
        let toml = json_to_toml(&json, true).unwrap();
        let res = toml.to_string();
        assert_eq!(res, "1.4");
    }
}
