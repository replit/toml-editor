use toml_edit::{Table, Item};
use anyhow::Result;

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
                    panic!("cannot set a key on a non-table")
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
                    panic!("Cannot add key to a non-table")
                }
            }
        }
    }
}

// #[cfg(test)]
// mod table_header_adder_tests {
//     #[test]
//     fn test_toby_thing() {
//         assert_eq!("abc", "[1, 2, 3]");
//     }
// }