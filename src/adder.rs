use anyhow::{bail, Context, Result};
use serde_json::{from_str, Value as JValue};
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

use crate::converter::json_to_toml;
use crate::field_finder::{get_field, DoInsert, TomlValue};

pub fn handle_add(field: &str, value: &str, doc: &mut DocumentMut) -> Result<()> {
    let mut path_split = field
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let last_field = path_split.pop().context("Path is empty")?;

    let final_field_value =
        get_field(&path_split, &last_field, DoInsert::Yes, doc).context("Could not find field")?;

    let field_value_json: JValue = from_str(value).context("parsing value field in add request")?;

    let is_inline = matches!(
        final_field_value,
        TomlValue::InlineTable(_) | TomlValue::Array(_) | TomlValue::Value(_)
    );

    let field_value_toml: Item = json_to_toml(&field_value_json, is_inline)
        .context("converting value in add request from json to toml")?;

    match final_field_value {
        TomlValue::Table(table) => add_in_table(table, &last_field, field_value_toml),
        TomlValue::ArrayOfTables(array) => {
            add_in_array_of_tables(array, &last_field, field_value_toml)
        }
        TomlValue::Array(array) => add_in_array(array, &last_field, field_value_toml),
        TomlValue::InlineTable(table) => add_in_inline_table(table, &last_field, field_value_toml),
        TomlValue::Value(value) => add_in_generic_value(value, &last_field, field_value_toml),
    }
}

fn add_in_table(table: &mut Table, last_field: &str, toml: Item) -> Result<()> {
    table.insert(last_field, toml);
    Ok(())
}

fn add_in_array_of_tables(array: &mut ArrayOfTables, last_field: &str, toml: Item) -> Result<()> {
    let insert_at_index = last_field.parse::<usize>().context("parsing last_field")?;

    let table = match toml {
        Item::Table(table) => table,
        _ => bail!("could not convert json to toml"),
    };

    if insert_at_index >= array.len() {
        array.push(table);
    } else {
        let table_to_modify = array
            .get_mut(insert_at_index)
            .context("getting table at index")?;
        *table_to_modify = table;
    }

    Ok(())
}

fn add_in_inline_table(table: &mut InlineTable, last_field: &str, toml: Item) -> Result<()> {
    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            table
                .insert(last_field, value)
                .context("could not insert value into inline table")?;
        }
        _ => bail!("could not convert json to inline toml"),
    }

    Ok(())
}

fn add_in_array(array: &mut Array, last_field: &str, toml: Item) -> Result<()> {
    let insert_at_index = last_field
        .parse::<usize>()
        .context("could not parse last_field as usize")?;

    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            if insert_at_index >= array.len() {
                array.push(value);
            } else {
                let value_to_modify = array
                    .get_mut(insert_at_index)
                    .context("could not get value at index")?;

                *value_to_modify = value;
            }
        }
        _ => bail!("could not convert json to toml"),
    }

    Ok(())
}

fn add_in_generic_value(generic_value: &mut Value, last_field: &str, toml: Item) -> Result<()> {
    match generic_value {
        Value::InlineTable(table) => add_in_inline_table(table, last_field, toml),
        Value::Array(array) => add_in_array(array, last_field, toml),
        _ => bail!("could not add into generic value"),
    }
}

#[cfg(test)]
mod adder_tests {
    use super::*;
    use toml_edit::{DocumentMut, TomlError};

    fn get_dotreplit_content_with_formatting() -> Result<DocumentMut, TomlError> {
        r#"test = "yo"
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
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
            .parse::<DocumentMut>()
    }

    macro_rules! add_test {
        ($name:ident, $field:expr, $value:expr, $contents:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut doc = $contents;
                let expected = $expected;
                let field = $field;
                let value = $value.to_string();

                let result = handle_add(field, &value, &mut doc);
                assert!(result.is_ok(), "error: {:?}", result);
                assert_eq!(doc.to_string().trim(), expected.trim());
            }
        };
    }

    add_test!(
        add_to_toml_basic,
        "new",
        "\"yo\"",
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
new = "yo"
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group"
[[foo.arr]]
        none = "all"
    "#
    );

    add_test!(
        add_to_toml_deep,
        "foo/bla/new",
        "\"yo\"",
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
[foo.bla]
    bro = 123
new = "yo"
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group"
[[foo.arr]]
        none = "all"
    "#
    );

    add_test!(
        add_array,
        "new",
        r#"["a", "b", "c"]"#,
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
new = ["a", "b", "c"]
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group"
[[foo.arr]]
        none = "all"
    "#
    );

    add_test!(
        add_array_at_index,
        "foo/arr/1/glub",
        r#"{"hi": 123}"#,
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]

[foo.arr.glub]
hi = 123
[[foo.arr]]
        none = "all"
    "#
    );

    add_test!(
        replace_large,
        "foo",
        r#"[1, 2, 3]"#,
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
foo = [1, 2, 3]
"#
    );

    add_test!(
        simple_push_into_array,
        "arr/2",
        "123",
        r#"arr = [1, 2]"#.parse::<DocumentMut>().unwrap(),
        r#"arr = [1, 2, 123]"#
    );

    add_test!(
        push_into_table_array,
        "foo/arr/3",
        r#"{}"#,
        get_dotreplit_content_with_formatting().unwrap(),
        r#"
test = "yo"
[foo]
  bar = "baz"  # comment
  inlineTable = {a = "b", c = "d" }
  inlineArray = [ "e", "f" ]
[foo.bla]
    bro = 123
[[foo.arr]]
    glub = "glub" # more comment
# comment here
# comment there

    [[foo.arr]]
        glub = "group"
[[foo.arr]]
        none = "all"

[[foo.arr]]
    "#
    );

    add_test!(
        add_as_you_traverse,
        "foo/arr",
        r#""yup""#,
        r#"meep = "nope""#.parse::<DocumentMut>().unwrap(),
        r#"meep = "nope"

[foo]
arr = "yup""#
    );

    add_test!(
        add_array_objects_deep,
        "foo/0/hi",
        r#"123"#,
        r#"top = "hi""#.parse::<DocumentMut>().unwrap(),
        r#"top = "hi"

[[foo]]
hi = 123
"#
    );

    add_test!(
        add_array_objects,
        "foo/0",
        r#"{"hi": 123}"#,
        r#"top = "hi""#.parse::<DocumentMut>().unwrap(),
        r#"top = "hi"

[[foo]]
hi = 123
"#
    );

    add_test!(
        add_another_array_object,
        "foo/1",
        r#"{"hi": 1234}"#,
        r#"top = "hi"
[[foo]]
hi = 123"#
            .parse::<DocumentMut>()
            .unwrap(),
        r#"top = "hi"
[[foo]]
hi = 123

[[foo]]
hi = 1234
"#
    );

    add_test!(
        preserve_ordering_on_unrelated_field,
        "run",
        r#""echo heyo""#,
        r#"
run = "echo hi"

[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
        .parse::<DocumentMut>()
        .unwrap(),
        r#"
run = "echo heyo"

[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
    );

    add_test!(
        preserve_ordering_on_semi_related_field,
        "env/TEST",
        r#""heyo""#,
        r#"
[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
        .parse::<DocumentMut>()
        .unwrap(),
        r#"
[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
TEST = "heyo"
"#
    );

    add_test!(
        preserve_ordering_on_related_field,
        "env/PATH",
        r#""${VIRTUAL_ENV}/bin""#,
        r#"
[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
        .parse::<DocumentMut>()
        .unwrap(),
        r#"
[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
    );

    add_test!(
        preserve_ordering_on_add_object,
        "env",
        r#"{"VIRTUAL_ENV":"/home/runner/${REPL_SLUG}/venv","PATH":"${VIRTUAL_ENV}/bin","PYTHONPATH":"${VIRTUAL_ENV}/lib/python3.8/site-packages","REPLIT_POETRY_PYPI_REPOSITORY":"https://package-proxy.replit.com/pypi/","MPLBACKEND":"TkAgg","POETRY_CACHE_DIR":"${HOME}/${REPL_SLUG}/.cache/pypoetry"}
"#,
        r#"
run = "hi"
"#
        .parse::<DocumentMut>()
        .unwrap(),
        r#"
run = "hi"

[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH = "${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY = "https://package-proxy.replit.com/pypi/"
MPLBACKEND = "TkAgg"
POETRY_CACHE_DIR = "${HOME}/${REPL_SLUG}/.cache/pypoetry"
"#
    );
}
