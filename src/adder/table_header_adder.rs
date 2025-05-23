use anyhow::{anyhow, bail, Context, Result};
use toml_edit::{array, Item, Table, Value};

/*
Perform an "add" at a table_header_path followed by a dotted_path.
Example:
table_header_path = "a/b"
dotted_path = "c/d"
value = "true"
yields:
```
[a.b]
c.b = true
```
*/
pub fn add_value_with_table_header_and_dotted_path(
    table: &mut Table,
    table_header_path: &[String],
    dotted_path: Option<Vec<String>>,
    value: Item,
    array_of_tables: bool,
    append_array_at_path: bool,
) -> Result<()> {
    match table_header_path.first() {
        None => {
            add_value_with_dotted_path(
                table,
                dotted_path.context("Missing 'path' value")?.as_slice(),
                value,
                append_array_at_path,
            )?;
            Ok(())
        }
        Some(field) => match table.get_mut(field) {
            Some(Item::Table(ref mut inner_table)) => {
                inner_table.set_dotted(table_header_path.len() > 1);
                add_value_with_table_header_and_dotted_path(
                    inner_table,
                    &table_header_path[1..],
                    dotted_path,
                    value,
                    array_of_tables,
                    append_array_at_path,
                )?;
                Ok(())
            }
            None | Some(Item::None) => {
                if table_header_path.len() > 1 || dotted_path.is_some() {
                    let mut inner_table = Table::new();
                    inner_table.set_dotted(table_header_path.len() > 1);
                    add_value_with_table_header_and_dotted_path(
                        &mut inner_table,
                        &table_header_path[1..],
                        dotted_path,
                        value,
                        array_of_tables,
                        append_array_at_path,
                    )?;
                    table.insert(field, Item::Table(inner_table));
                } else {
                    match value {
                        Item::Value(Value::InlineTable(it)) => {
                            if array_of_tables {
                                table.insert(field, array());
                                let aot = table[field].as_array_of_tables_mut().unwrap();
                                aot.push(it.into_table());
                            } else {
                                table.insert(field, Item::Table(it.into_table()));
                            }
                        }
                        other => {
                            bail!("unexpected value: {:?}", other);
                        }
                    }
                }
                Ok(())
            }
            Some(Item::Value(_)) => {
                bail!("cannot set a key on a non-table")
            }
            Some(Item::ArrayOfTables(aot)) => {
                match value {
                    Item::Value(Value::InlineTable(it)) => {
                        if array_of_tables {
                            aot.push(it.into_table());
                        } else {
                            bail!("Expected [[]] syntax for appending to an array of tables");
                        }
                    }
                    other => {
                        bail!("unexpected value: {:?}", other);
                    }
                };
                Ok(())
            }
        },
    }
}

/*
Perform an "add" at a dotted_path.
Example:
dotted_path = "a/b"
value = "true"
yields:
```
a.b = true
```
*/
fn add_value_with_dotted_path(
    table: &mut Table,
    dotted_path: &[String],
    value: Item,
    append_array_at_path: bool,
) -> Result<()> {
    match dotted_path.first() {
        None => Ok(()),
        Some(field) => match table.get_mut(field) {
            None | Some(Item::None) => {
                if dotted_path.len() > 1 {
                    let mut inner_table = Table::new();
                    inner_table.set_dotted(true);
                    add_value_with_dotted_path(
                        &mut inner_table,
                        &dotted_path[1..],
                        value,
                        append_array_at_path,
                    )?;
                    table.insert(field, Item::Table(inner_table));
                    Ok(())
                } else if append_array_at_path {
                    let mut arr = toml_edit::Array::new();
                    arr.push(
                        value
                            .into_value()
                            .map_err(|_| anyhow!("Cannot append non-value item to array"))?,
                    );
                    table.insert(field, Item::Value(Value::Array(arr)));
                    Ok(())
                } else {
                    table.insert(field, value);
                    Ok(())
                }
            }
            Some(Item::Table(ref mut inner_table)) => {
                if dotted_path.len() > 1 {
                    inner_table.set_dotted(true);
                    add_value_with_dotted_path(
                        inner_table,
                        &dotted_path[1..],
                        value,
                        append_array_at_path,
                    )
                } else {
                    table.insert(field, value);
                    Ok(())
                }
            }
            Some(item @ Item::Value(_)) => {
                if dotted_path.len() != 1 {
                    bail!("Cannot overwrite a non-table with a table")
                }

                if append_array_at_path {
                    let arr = item
                        .as_array_mut()
                        .context(format!("Cannot append non-array field '{field}'"))?;
                    arr.push(
                        value
                            .into_value()
                            .map_err(|_| anyhow!("Cannot append non-value item to array"))?,
                    );
                } else {
                    table.insert(field, value);
                }
                Ok(())
            }
            Some(Item::ArrayOfTables(_)) => {
                bail!("Cannot add key to a array of tables")
            }
        },
    }
}
