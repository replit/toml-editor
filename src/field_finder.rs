use std::{io::Error, io::ErrorKind};
// use toml_edit::{array, table, value, Array, Document, Item, Value};
use toml_edit::{Document, Item, Value, value, Table};

// pub fn get_or_insert_table_mut<'a>(
//     doc: &'a mut Document,
//     table_path: &[String],
// ) -> Result<&'a mut toml_edit::Item, Error> {
//     get_table_mut_internal(doc, table_path, true)
// } 

pub fn get_feild<'a>(
    doc: &'a mut Document,
    table_path: &[String],
    insert_if_not_exists: bool,
) -> Result<&'a mut Item, Error> {
    /// Descend into the toml tree until the table is found
    fn descend<'a>(
        input: &'a mut Item,
        path: &[String],
        insert_if_not_exists: bool,
    ) -> Result<&'a mut Item, Error> {
        if let Some(segment) = path.get(0) {
            match input {
                Item::Table(table) => {
                    let val = if insert_if_not_exists {
                        table[&segment].or_insert(toml_edit::table())
                    } else {
                        match table.get_mut(&segment) {
                            Some(val) => val,
                            None => return Err(Error::new(
                                ErrorKind::NotFound,
                                format!("Could not find table {}", segment),
                            )),
                        }
                    };

                    descend(val, &path[1..], insert_if_not_exists)
                },
                Item::Value(value) => {
                    match value {
                        Value::Array(array) => {
                            let array_index = match segment.parse::<usize>() {
                                Ok(index) => index,
                                Err(_) => return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Could not parse {} as an array index", segment),
                                )),
                            };

                            let val = match array.get_mut(array_index) {
                                Some(val) => val,
                                None => return Err(Error::new(
                                    ErrorKind::NotFound,
                                    format!("Could not find array index {}", segment),
                                )),
                            };

                            /// HERE IS THE ISSUE
                            descend(val, &path[1..], insert_if_not_exists)
                        },
                        Value::InlineTable(table) => {
                        },
                        _ => return Err(Error::new(
                            ErrorKind::NotFound,
                            format!("Unsupported value format"),
                        )),
                    }
                },
                Item::ArrayOfTables(array) => {


                }
            }
            // let value = if insert_if_not_exists {
            //     input[&segment].or_insert(toml_edit::table())
            // } else {
            //     match input.get_mut(&segment) {
            //         Some(value) => value,
            //         None => return Err(Error::new(ErrorKind::NotFound, "")),
            //     }
            // };

            // if value.is_table_like() {
            //     descend(value, &path[1..], insert_if_not_exists)
            // } else {
            //     Err(Error::new(ErrorKind::Other, "Non existant table"))
            // }
        } else {
            Ok(input)
        }
    }

    descend(doc.as_item_mut(), table_path, insert_if_not_exists)
}

// pub fn get_field<'a>(fields_path: Vec<&str>, doc: &'a mut Document) -> Result<&'a mut Item, Error> {
//     // the field path is seperated by '/' 
//     // we need to iterate through each field in the doc
//     // we need to return the last field in the path

//     let mut fields = fields_path.clone();

//     println!("fields: {:?}", fields.len());

//     if fields.len() < 1 {
//         return Err(Error::new(ErrorKind::Other, "Field path is empty in get_field"));
//     }

//     // we need to get first field to get started
//     // we can directly grab like this since top level toml
//     // is never an array and always a table
//     let start_field_res = doc.get_mut(fields.remove(0));
//     if start_field_res.is_none() {
//         return Err(Error::new(
//             ErrorKind::Other,
//             "error: could not find field",
//         ));
//     }
//     let mut current_item = start_field_res.unwrap();

//     for field in fields {
//         // after the first level, we need to check if the current item is
//         // an array or a table
//         match current_item {
//             Item::Table(table) => {
//                 current_item = match table.get_mut(field) {
//                     Some(item) => item,
//                     None => return Err(Error::new(
//                         ErrorKind::Other,
//                         "error: could not find field",
//                     )),
//                 };
//             },
//             Item::Value(val) => {
//                 match val {
//                     Value::Array(array) => {
//                         let array_index = match field.parse::<usize>() {
//                             Ok(index) => index,
//                             Err(_) => return Err(Error::new(ErrorKind::Other, "could not parse array index")),
//                         };

//                         if array_index >= array.len() {
//                             return Err(Error::new(
//                                 ErrorKind::Other,
//                                 "error: array index out of bounds",
//                             ));
//                         }

                        
//                         let array_item: &mut Value = match array.get_mut(array_index) {
//                             Some(item) => item,
//                             None => return Err(Error::new(
//                                 ErrorKind::Other,
//                                 "error: array index out of bounds",
//                             )),
//                         };

//                         current_item = &mut Item::Value(array_item);
//                     },
//                     Value::InlineTable(table) => {
//                         current_item = match table.get_mut(field) {
//                             Some(item) => item,
//                             None => return Err(Error::new(
//                                 ErrorKind::Other,
//                                 "error: could not find field",
//                             )),
//                         };
//                     },
//                     _ => {
//                         return Err(Error::new(
//                             ErrorKind::Other,
//                             "error: field is not recursable",
//                         ));
//                     }
//                 }
//             },
//             Item::ArrayOfTables(array) => {
//                 let array_index = match field.parse::<usize>() {
//                     Ok(index) => index,
//                     Err(_) => return Err(Error::new(ErrorKind::Other, "could not parse array index")),
//                 };

//                 if array_index >= array.len() {
//                     return Err(Error::new(
//                         ErrorKind::Other,
//                         "error: array index out of bounds",
//                     ));
//                 }

//                 current_item = match array.get_mut(array_index) {
//                     Some(t) => &mut Item::Table(t),
//                     None => return Err(Error::new(
//                         ErrorKind::Other,
//                         "error: could not find field",
//                     )),
//                 };
//             },
//             _ => {
//                 return Err(Error::new(ErrorKind::Other, "Field path is invalid"));
//             }
//         }
//     }

//     Ok(current_item)
// }

// fn recurse_on_value(value: &mut Value) {
//     match value {
//         Value::InlineTable(table) => {
//             for (key, val) in table.iter_mut() {
//                 recurse_on_value(val);
//             }
//         },
//         Value::Array(array) => {
//             for val in array.iter_mut() {
//                 recurse_on_value(val);
//             }
//         },
//         _ => {},
//     }
// }


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_field_test() {
//         let doc_string = r#"
// test = "yo"
// [foo]
// bar = "baz"
// [foo.bla]
// bla = "bla"
// "#;

//         let doc_res = doc_string.parse::<Document>();
//         if doc_res.is_err() {
//             println!("error: could not parse .replit");
//             return;
//         }
//         let mut doc = doc_res.unwrap();

//         let field_res = get_field("test".to_string(), &mut doc);
//         if field_res.is_err() {
//             println!("error: could not get field");
//             return;
//         }
//         let field = field_res.unwrap();

//         assert_eq!(field.to_string(), "yo");

//         let field_res = get_field("foo/bar".to_string(), &mut doc);
//         if field_res.is_err() {
//             println!("error: could not get field");
//             return;
//         }
//         let field = field_res.unwrap();

//         assert_eq!(field.to_string(), "baz");
//     }
// }

