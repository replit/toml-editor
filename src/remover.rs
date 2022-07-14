use crate::field_finder::{get_field, TomlValue};
use std::{io::Error, io::ErrorKind};
use toml_edit::{Array, ArrayOfTables, Document, InlineTable, Table};

pub fn handle_remove(field: String, doc: &mut Document) -> Result<(), Error> {
    let mut path_split = field
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let last_field = match path_split.pop() {
        Some(last_field) => last_field,
        None => return Err(Error::new(ErrorKind::Other, "Path is empty")),
    };

    let insert_if_not_exists = false;
    let field = match get_field(&path_split, &last_field, insert_if_not_exists, doc) {
        Ok(field) => field,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                // if you can't find the field then it's already gone
                // so we don't need to remove it or do anything else
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };

    match field {
        TomlValue::Table(table) => remove_in_table(table, last_field),
        TomlValue::Array(array) => remove_in_array(array, last_field),
        TomlValue::ArrayOfTables(array) => remove_in_array_of_tables(array, last_field),
        TomlValue::InlineTable(table) => remove_in_inline_table(table, last_field),
        TomlValue::Value(_) => Err(Error::new(
            ErrorKind::Other,
            "error: cannot remove_in non array/table value",
        )),
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

#[cfg(test)]
mod remover_tests {
    use super::*;
    use toml_edit::{Document, TomlError};

    fn get_dotreplit_content() -> Result<Document, TomlError> {
        r#"test = "yo"
[foo]
bar = "baz"
[foo.bla]
bro = 123
[[foo.arr]]
glub = "glub"
[[foo.arr]]
glub = "group"
[[foo.arr]]
none = "all""#
            .to_string()
            .parse::<Document>()
    }

    fn get_dotreplit_content_with_formatting() -> Result<Document, TomlError> {
        r#"test = "yo"
[foo]
  bar = "baz"  # comment
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group"
[[foo.arr]]
        none = "all""#
            .to_string()
            .parse::<Document>()
    }

    macro_rules! remove_test {
        ($name:ident, $field:expr, $contents:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut doc = $contents;
                handle_remove($field.to_string(), &mut doc).unwrap();
                assert_eq!(doc.to_string().trim(), $expected.trim());
            }
        };
    }

    remove_test!(
        test_remove_basic,
        "foo/bar",
        get_dotreplit_content().unwrap(),
        r#"
test = "yo"
[foo]
[foo.bla]
bro = 123
[[foo.arr]]
glub = "glub"
[[foo.arr]]
glub = "group"
[[foo.arr]]
none = "all""#
    );

    remove_test!(
        test_remove_array_index,
        "foo/arr/0",
        get_dotreplit_content().unwrap(),
        r#"
test = "yo"
[foo]
bar = "baz"
[foo.bla]
bro = 123
[[foo.arr]]
glub = "group"
[[foo.arr]]
none = "all""#
    );

    remove_test!(
        test_remove_shallow,
        "foo",
        get_dotreplit_content().unwrap(),
        r#"test = "yo""#
    );

    remove_test!(
        test_remove_deep,
        "foo/arr/2/none",
        get_dotreplit_content().unwrap(),
        r#"
test = "yo"
[foo]
bar = "baz"
[foo.bla]
bro = 123
[[foo.arr]]
glub = "glub"
[[foo.arr]]
glub = "group"
[[foo.arr]]"#
    );

    remove_test!(
        test_keep_comments,
        "foo/arr/2",
        get_dotreplit_content_with_formatting().unwrap(),
        r#"test = "yo"
[foo]
  bar = "baz"  # comment
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group""#
    );

    remove_test!(
        test_remove_inline_array,
        "arr/1",
        "arr = [1, 2, 3, 4] # comment".parse::<Document>().unwrap(),
        "arr = [1, 3, 4] # comment"
    );

    remove_test!(
        test_remove_inline_table,
        "tbl/bla",
        "tbl = { bla = \"bar\", blue = 123 } # go go"
            .parse::<Document>()
            .unwrap(),
        "tbl = { blue = 123 } # go go"
    );

    remove_test!(
        test_remove_missing_early,
        "foo/bar/baz/boop",
        "[foo]
        yup = 123"
            .parse::<Document>()
            .unwrap(),
        "[foo]
        yup = 123"
    );
}
