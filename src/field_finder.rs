use std::{io::Error, io::ErrorKind};

use anyhow::{bail, Context, Result};
use toml_edit::{Array, ArrayOfTables, Document, InlineTable, Item, Table, Value};

pub enum TomlValue<'a> {
    Table(&'a mut Table),
    Array(&'a mut Array),
    Value(&'a mut Value),
    InlineTable(&'a mut InlineTable),
    ArrayOfTables(&'a mut ArrayOfTables),
}

#[derive(PartialEq, Eq)]
pub enum DoInsert {
    Yes,
    No,
}

pub fn get_field<'a>(
    path: &[String],
    last_field: &String,
    do_insert: DoInsert,
    doc: &'a mut Document,
) -> Result<TomlValue<'a>> {
    let current_table = doc.as_table_mut();

    descend_table(current_table, path, do_insert, last_field)
}

fn descend_table<'a>(
    table: &'a mut Table,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    let segment = match path.get(0) {
        Some(segment) => segment,
        None => return Ok(TomlValue::Table(table)),
    };

    let val = match do_insert {
        DoInsert::Yes => {
            // if next segment exists and is an integer insert array of tables
            let insert_array_of_tables = match path.get(1) {
                Some(segment) => segment.parse::<usize>().is_ok(),
                None => last_field.parse::<usize>().is_ok(),
            };

            let to_insert_as_backup = if insert_array_of_tables {
                toml_edit::array()
            } else {
                toml_edit::table()
            };

            table[segment].or_insert(to_insert_as_backup)
        }
        DoInsert::No => table
            .get_mut(segment)
            .ok_or(Error::new(ErrorKind::NotFound, "Path does not exist"))?,
    };

    descend_item(val, &path[1..], do_insert, last_field)
}

fn descend_item<'a>(
    item: &'a mut Item,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    match item {
        Item::Table(table) => descend_table(table, path, do_insert, last_field),
        Item::Value(value) => descend_value(value, path, do_insert, last_field),
        Item::ArrayOfTables(array) => descend_array_of_tables(array, path, do_insert, last_field),
        _ => bail!("Unsupported item format"),
    }
}

fn descend_value<'a>(
    value: &'a mut Value,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    match value {
        Value::Array(array) => descend_array(array, path, do_insert, last_field),
        Value::InlineTable(table) => descend_inline_table(table, path, do_insert, last_field),
        _ => {
            if !path.is_empty() {
                bail!("Adding into unsupported generic value")
            }

            // if no more path, then this is the last item that we want
            Ok(TomlValue::Value(value))
        }
    }
}

fn descend_array_of_tables<'a>(
    array: &'a mut ArrayOfTables,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    let segment = match path.get(0) {
        Some(segment) => segment,
        None => return Ok(TomlValue::ArrayOfTables(array)),
    };

    let array_index = segment
        .parse::<usize>()
        .context("Could not parse segment as array index")?;
    // if array index is one larger than the current array length, then we need to add a new table
    if array_index == array.len() {
        if do_insert == DoInsert::No {
            bail!(Error::new(ErrorKind::NotFound, "Path does not exist"));
        }

        array.push(Table::new());
    }

    let tbl = array
        .get_mut(array_index)
        .context("Could not find array index")?;

    descend_table(tbl, &path[1..], do_insert, last_field)
}

fn get_last_field_container(last_field: &str) -> Value {
    // if last field is a number, then we need to create an array
    if last_field.parse::<usize>().is_ok() {
        Value::Array(Array::new())
    // if last field is a string, then we need to create a table
    } else {
        Value::InlineTable(InlineTable::new())
    }
}

fn descend_inline_table<'a>(
    inline_table: &'a mut InlineTable,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    let segment = match path.get(0) {
        Some(segment) => segment,
        None => return Ok(TomlValue::InlineTable(inline_table)),
    };

    if do_insert == DoInsert::Yes && !inline_table.contains_key(segment) {
        inline_table.insert(segment, get_last_field_container(last_field));
    }

    let val = inline_table
        .get_mut(segment)
        .ok_or(Error::new(ErrorKind::NotFound, "Path does not exist"))?;

    descend_value(val, &path[1..], do_insert, last_field)
}

fn descend_array<'a>(
    array: &'a mut Array,
    path: &[String],
    do_insert: DoInsert,
    last_field: &String,
) -> Result<TomlValue<'a>> {
    let segment = match path.get(0) {
        Some(segment) => segment,
        None => return Ok(TomlValue::Array(array)),
    };

    let array_index = segment
        .parse::<usize>()
        .context("Could not parse segment as array index")?;

    if array_index == array.len() {
        if do_insert == DoInsert::No {
            bail!(Error::new(ErrorKind::NotFound, "Path does not exist"));
        }

        array.push(get_last_field_container(last_field));
    }

    let val = array
        .get_mut(array_index)
        .context("Could not find array index")?;

    descend_value(val, &path[1..], do_insert, last_field)
}

#[cfg(test)]
mod finger_tests {
    use super::*;
    use toml_edit::Document;

    #[test]
    fn get_basic_field() {
        let doc_string = r#"
test = "yo"
[foo]
bar = "baz"
[foo.bla]
bla = "bla"
"#;

        let mut doc = doc_string.parse::<Document>().unwrap();
        let val = get_field(
            &(vec!["foo".to_string()]),
            &"bar".to_string(),
            DoInsert::Yes,
            &mut doc,
        )
        .unwrap();

        if let TomlValue::Table(table) = val {
            assert!(table.contains_key("bar"));
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn insert_inline_array() {
        let doc_string = r#"test = [ 1 ]"#;
        let mut doc = doc_string.parse::<Document>().unwrap();
        let fields = ["test".to_string()];
        let val = get_field(&(fields), &"1".to_string(), DoInsert::Yes, &mut doc).unwrap();

        if let TomlValue::Array(array) = val {
            assert_eq!(array.len(), 1);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn insert_table_array() {
        let doc_string = r#"
[[test]]
foo = "bar"
[[test]]
foo = "baz"
"#;
        let mut doc = doc_string.parse::<Document>().unwrap();
        let fields = ["test".to_string()];
        let val = get_field(&(fields), &"2".to_string(), DoInsert::Yes, &mut doc).unwrap();

        if let TomlValue::ArrayOfTables(array) = val {
            assert_eq!(array.len(), 2);
        } else {
            panic!("Expected array");
        }
    }
}
