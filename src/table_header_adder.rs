use toml_edit::{Table, Item};
use anyhow::{bail, Result};

pub fn add_value_with_table_header_and_dotted_path(
    table: &mut Table,
    table_header_path: &[String],
    dotted_path: &[String],
    value: Item) -> Result<()>
{
    match table_header_path.get(0) {
        None => add_value_with_dotted_path(table, dotted_path, value),
        Some(field) => {
            match table.get_mut(field) {
                Some(Item::Table(ref mut inner_table)) => {
                    inner_table.set_dotted(table_header_path.len() > 1);
                    _ = add_value_with_table_header_and_dotted_path(
                        inner_table,
                        &table_header_path[1..],
                        dotted_path,
                        value
                    );
                    Ok(())
                }
                None => {
                    let mut inner_table = Table::new();
                    inner_table.set_dotted(table_header_path.len() > 1);
                    _ = add_value_with_table_header_and_dotted_path(
                        &mut inner_table,
                        &table_header_path[1..],
                        dotted_path,
                        value
                    );
                    table.insert(field, Item::Table(inner_table));
                    Ok(())
                }
                Some(_) => {
                    bail!("cannot set a key on a non-table")
                }
            }
        }
    }
}

fn add_value_with_dotted_path(table: &mut Table, dotted_path: &[String], value: Item) -> Result<()> {
    match dotted_path.get(0) {
        None => {
            Ok(())
        },
        Some(field) => {
            match table.get_mut(field) {
                None => {
                    if dotted_path.len() > 1 {
                        let mut inner_table = Table::new();
                        inner_table.set_dotted(true);
                        _ = add_value_with_dotted_path(
                            &mut inner_table,
                            &dotted_path[1..],
                            value
                        );
                        table.insert(field, Item::Table(inner_table));
                    } else {
                        table.insert(field, value);
                    }
                    Ok(())
                }
                Some(Item::Table(ref mut inner_table)) => {
                    if dotted_path.len() > 1 {
                        inner_table.set_dotted(true);
                        _ = add_value_with_dotted_path(
                            inner_table,
                            &dotted_path[1..],
                            value
                        );
                    } else {
                        table.insert(field, value);
                    }
                    Ok(())
                }
                Some(Item::Value(_)) => {
                    table.insert(field, value);
                    Ok(())
                }
                Some(_) => {
                    bail!("Cannot add key to a non-table")
                }
            }
        }
    }
}

#[cfg(test)]
mod table_header_adder_tests {
    use super::*;
    use toml_edit::{DocumentMut, value};

    #[test]
    fn test_one_element_table_header() {
        let mut doc = "".to_string().parse::<DocumentMut>()
            .expect("invalid doc");
        let table_header_path = vec!["moduleConfig"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let dotted_path = vec!["interpreters", "ruby", "enable"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        _ = add_value_with_table_header_and_dotted_path(&mut doc, &table_header_path, &dotted_path, value(true));
        assert_eq!(doc.to_string(), "[moduleConfig]\ninterpreters.ruby.enable = true\n");
    }

    #[test]
    fn test_two_element_table_header() {
        let mut doc = "".to_string().parse::<DocumentMut>()
            .expect("invalid doc");
        let table_header_path = vec!["moduleConfig", "interpreters"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let dotted_path = vec!["ruby", "enable"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        _ = add_value_with_table_header_and_dotted_path(&mut doc, &table_header_path, &dotted_path, value(true));
        assert_eq!(doc.to_string(), "[moduleConfig.interpreters]\nruby.enable = true\n");
    }

    #[test]
    fn test_preserve_existing() {
        let mut doc = r#"
[moduleConfig]
bundles.go.enable = true
        "#.to_string().parse::<DocumentMut>()
            .expect("invalid doc");
        let table_header_path = vec!["moduleConfig"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let dotted_path = vec!["interpreters", "ruby", "enable"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        _ = add_value_with_table_header_and_dotted_path(&mut doc, &table_header_path, &dotted_path, value(true));
        assert_eq!(doc.to_string(), "\n[moduleConfig]\nbundles.go.enable = true\ninterpreters.ruby.enable = true\n        ");
    }

    #[test]
    fn test_preserve_existing_inner_tables() {
        let mut doc = r#"
[moduleConfig]
interpreter.ruby.enable = true
        "#.to_string().parse::<DocumentMut>()
            .expect("invalid doc");
        let table_header_path = vec!["moduleConfig"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let dotted_path = vec!["interpreters", "ruby", "version"]
            .iter().map(|s| s.to_string()).collect::<Vec<String>>();
        _ = add_value_with_table_header_and_dotted_path(&mut doc, &table_header_path, &dotted_path, value("3.2.3"));
        assert_eq!(doc.to_string(), "\n[moduleConfig]\ninterpreter.ruby.enable = true\ninterpreters.ruby.version = \"3.2.3\"\n        ");
    }
}