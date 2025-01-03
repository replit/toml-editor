use anyhow::{bail, Result};
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
) -> Result<()> {
    match table_header_path.get(0) {
        None => {
            add_value_with_dotted_path(
                table,
                dotted_path.expect("Missing 'path' value").as_slice(),
                value,
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
) -> Result<()> {
    match dotted_path.get(0) {
        None => Ok(()),
        Some(field) => match table.get_mut(field) {
            None | Some(Item::None) => {
                if dotted_path.len() > 1 {
                    let mut inner_table = Table::new();
                    inner_table.set_dotted(true);
                    return add_value_with_dotted_path(&mut inner_table, &dotted_path[1..], value)
                        .map(|_| table.insert(field, Item::Table(inner_table)))
                        .map(|_| ());
                } else {
                    table.insert(field, value);
                    Ok(())
                }
            }
            Some(Item::Table(ref mut inner_table)) => {
                if dotted_path.len() > 1 {
                    inner_table.set_dotted(true);
                    return add_value_with_dotted_path(inner_table, &dotted_path[1..], value);
                } else {
                    table.insert(field, value);
                    Ok(())
                }
            }
            Some(Item::Value(_)) => {
                if dotted_path.len() == 1 {
                    table.insert(field, value);
                    Ok(())
                } else {
                    bail!("Cannot overwrite a non-table with a table")
                }
            }
            Some(Item::ArrayOfTables(_)) => {
                bail!("Cannot add key to a array of tables")
            }
        },
    }
}

#[cfg(test)]
mod table_header_adder_tests {
    use super::*;
    use toml_edit::{value, DocumentMut, Formatted, InlineTable};

    macro_rules! meta_add_test {
        ($name:ident, $table_header:expr, $path:expr, $value:expr, $contents:expr, $expected:expr, $result:ident, $($assertion:stmt)*) => {
            #[test]
            fn $name() {
                let mut doc = $contents.to_string().parse::<DocumentMut>().expect("invalid doc");

                let mut table_header_path = $table_header
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                // We need to be explicit to guide None towards a compatible type:
                let dotted_path = ($path as Option<Vec<&str>>).map(|path|
                    path
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>());

                let array_of_tables = if table_header_path.last().is_some_and(|k| k == "[[]]") {
                    table_header_path.pop();
                    true
                } else {
                    false
                };
                let $result = add_value_with_table_header_and_dotted_path(
                    &mut doc,
                    &table_header_path,
                    dotted_path,
                    $value,
                    array_of_tables,
                );
                $(
                    $assertion
                )*
                assert_eq!(
                    doc.to_string().trim(),
                    $expected.trim(),
                );
            }
        };
    }

    macro_rules! add_test {
        ($name:ident, $table_header:expr, $path:expr, $value:expr, $contents:expr, $expected:expr) => {
            meta_add_test!(
                $name,
                $table_header,
                $path,
                $value,
                $contents,
                $expected,
                result,
                {
                    assert!(result.is_ok(), "error: {:?}", result);
                }
            );
        };
    }

    macro_rules! add_error_test {
        ($name:ident, $table_header:expr, $path:expr, $value:expr, $contents:expr, $expected:expr) => {
            meta_add_test!(
                $name,
                $table_header,
                $path,
                $value,
                $contents,
                $expected,
                result,
                {
                    assert!(result.is_err(), "expected an error, got : {:?}", result);
                }
            );
        };
    }

    add_test!(
        test_one_element_table_header,
        vec!["moduleConfig"],
        Some(vec!["interpreters", "ruby", "enable"]),
        value(true),
        "",
        r#"
[moduleConfig]
interpreters.ruby.enable = true
        "#
    );

    add_test!(
        test_two_element_table_header,
        vec!["moduleConfig", "interpreters"],
        Some(vec!["ruby", "enable"]),
        value(true),
        "",
        r#"
[moduleConfig.interpreters]
ruby.enable = true
        "#
    );

    add_test!(
        test_preserve_existing,
        vec!["moduleConfig"],
        Some(vec!["interpreters", "ruby", "enable"]),
        value(true),
        r#"
[moduleConfig]
bundles.go.enable = true
        "#,
        r#"
[moduleConfig]
bundles.go.enable = true
interpreters.ruby.enable = true
"#
    );

    add_test!(
        test_preserve_existing_inner_tables,
        vec!["moduleConfig"],
        Some(vec!["interpreters", "ruby", "version"]),
        value("3.2.3"),
        r#"
[moduleConfig]
interpreter.ruby.enable = true
        "#,
        r#"
[moduleConfig]
interpreter.ruby.enable = true
interpreters.ruby.version = "3.2.3"
        "#
    );

    add_error_test!(
        test_error_when_adding_key_to_non_table,
        vec!["moduleConfig"],
        Some(vec!["interpreters", "ruby", "version"]),
        value("3.2.3"),
        r#"
[moduleConfig]
interpreters.ruby = "my dear"
        "#,
        r#"
[moduleConfig]
interpreters.ruby = "my dear"
        "#
    );

    add_test!(
        test_add_arrays_of_tables,
        vec!["tool", "uv", "index", "[[]]"],
        None,
        {
            let mut it = InlineTable::default();
            it.insert("key", Value::String(Formatted::new("value".to_owned())));
            Item::Value(Value::InlineTable(it))
        },
        "",
        r#"
[[tool.uv.index]]
key = "value"
"#
    );

    add_test!(
        test_append_arrays_of_tables,
        vec!["tool", "uv", "index", "[[]]"],
        None,
        {
            let mut it = InlineTable::default();
            it.insert("key", Value::String(Formatted::new("second".to_owned())));
            Item::Value(Value::InlineTable(it))
        },
        r#"
[[tool.uv.index]]
key = "first"
        "#,
        r#"
[[tool.uv.index]]
key = "first"

[[tool.uv.index]]
key = "second"
"#
    );
}
