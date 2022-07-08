use crate::converter::json_to_toml;
use crate::field_finder::{get_field, TomlValue};
use serde_json::{from_str, Value as JValue};
use std::{io::Error, io::ErrorKind};
use toml_edit::{Array, ArrayOfTables, Document, InlineTable, Item, Table, Value};

pub fn handle_add(
    field: String,
    value_opt: Option<String>,
    doc: &mut Document,
) -> Result<(), Error> {
    let mut path_split = field
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let last_field = match path_split.pop() {
        Some(last_field) => last_field,
        None => return Err(Error::new(ErrorKind::Other, "Path is empty")),
    };

    let final_field_value = match get_field(&path_split, &last_field, doc) {
        Ok(final_field) => final_field,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Could not find field")),
    };

    if value_opt.is_none() {
        return Err(Error::new(
            ErrorKind::Other,
            "Expected value to be none null",
        ));
    }
    let value = value_opt.unwrap();

    let field_value_json: JValue = match from_str(value.as_str()) {
        Ok(json_field_value) => json_field_value,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: value field in add request is not json",
            ));
        }
    };

    let is_inline = match final_field_value {
        TomlValue::ArrayOfTables(_) => false,
        TomlValue::Table(_) => false,
        TomlValue::Array(_) => true,
        TomlValue::InlineTable(_) => true,
        TomlValue::Value(_) => true,
    };

    let field_value_toml: Item = match json_to_toml(&field_value_json, is_inline) {
        Ok(toml_field_value) => toml_field_value,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: value field in add request cannot be converted to toml",
            ));
        }
    };

    match final_field_value {
        TomlValue::Table(table) => add_in_table(table, last_field, field_value_toml),
        TomlValue::ArrayOfTables(array) => {
            add_in_array_of_tables(array, last_field, field_value_toml)
        }
        TomlValue::Array(array) => add_in_array(array, last_field, field_value_toml),
        TomlValue::InlineTable(table) => add_in_inline_table(table, last_field, field_value_toml),
        TomlValue::Value(value) => add_in_generic_value(value, last_field, field_value_toml),
    }
}

fn add_in_table(table: &mut Table, last_field: String, toml: Item) -> Result<(), Error> {
    table.insert(last_field.as_str(), toml);
    Ok(())
}

fn add_in_array_of_tables(
    array: &mut ArrayOfTables,
    last_field: String,
    toml: Item,
) -> Result<(), Error> {
    let insert_at_index = match last_field.parse::<usize>() {
        Ok(index) => index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse last_field as usize",
            ));
        }
    };

    match toml {
        Item::Table(table) => {
            if insert_at_index >= array.len() {
                array.push(table);
            } else {
                let table_to_modify = match array.get_mut(insert_at_index) {
                    Some(table) => table,
                    None => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "error: could not get mutable reference to table at index",
                        ));
                    }
                };

                *table_to_modify = table;
            }
        }
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    }

    Ok(())
}

fn add_in_inline_table(
    table: &mut InlineTable,
    last_field: String,
    toml: Item,
) -> Result<(), Error> {
    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            match table.insert(last_field.as_str(), value) {
                Some(_) => {}
                None => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "error: could not insert value into inline table",
                    ))
                }
            };
        }
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to inline toml",
            ));
        }
    }

    Ok(())
}

fn add_in_array(array: &mut Array, last_field: String, toml: Item) -> Result<(), Error> {
    let insert_at_index = match last_field.parse::<usize>() {
        Ok(index) => index,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not parse last_field as usize",
            ));
        }
    };

    // since we requested inline toml, this should be a value
    match toml {
        Item::Value(value) => {
            if insert_at_index >= array.len() {
                array.push(value);
            } else {
                let value_to_modify = match array.get_mut(insert_at_index) {
                    Some(value) => value,
                    None => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "error: could not get mutable reference to value at index",
                        ));
                    }
                };

                *value_to_modify = value;
            }
        }
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "error: could not convert json to toml",
            ));
        }
    }

    Ok(())
}

fn add_in_generic_value(
    generic_value: &mut Value,
    last_field: String,
    toml: Item,
) -> Result<(), Error> {
    match generic_value {
        Value::InlineTable(table) => add_in_inline_table(table, last_field, toml),
        Value::Array(array) => add_in_array(array, last_field, toml),
        _ => Err(Error::new(
            ErrorKind::Other,
            "error: could not add into generic value",
        )),
    }
}

#[cfg(test)]
mod adder_tests {
    use super::*;
    use toml_edit::{Document, TomlError};

    fn get_dotreplit_content_with_formatting() -> Result<Document, TomlError> {
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
            .parse::<Document>()
    }

    fn python_dotreplit() -> Result<Document, TomlError> {
        r#"# The command that runs the program. Commented out because it is not run when the interpreter command is set
# run = ["python3", "main.py"]
# The primary language of the repl. There can be others, though!
language = "python3"
entrypoint = "main.py"
hidden = ["venv", ".config", "**/__pycache__", "**/.mypy_cache", "**/*.pyc"]
audio = true

# Specifies which nix channel to use when building the environment.
[nix]
channel = "stable-21_11"

# Per-language configuration: python3
[languages.python3]
# Treats all files that end with `.py` as Python.
pattern = "**/*.py"
# Tells the workspace editor to syntax-highlight these files as
# Python.
syntax = "python"

  # The command needed to start the Language Server Protocol. For
  # linting and formatting.
  [languages.python3.languageServer]
  start = ["pylsp"]

# The command to start the interpreter.
[interpreter]
  [interpreter.command]
  args = [
    "stderred",
    "--",
    "prybar-python3",
    "-q",
    "--ps1",
    "\u0001\u001b[33m\u0002\u0001\u001b[00m\u0002 ",
    "-i",
  ]
  env = { LD_LIBRARY_PATH = "$PYTHON_LD_LIBRARY_PATH" }

[env]
MPLBACKEND = "TkAgg"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH = "${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY = "https://package-proxy.replit.com/pypi/"
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"

# Enable unit tests. This is only supported for a few languages.
[unitTest]
language = "java"

# Add a debugger!
[debugger]
support = true

  # How to start the debugger.
  [debugger.interactive]
  transport = "localhost:0"
  startCommand = ["dap-python", "main.py"]

    # How to communicate with the debugger.
    [debugger.interactive.integratedAdapter]
    dapTcpAddress = "localhost:0"

    # How to tell the debugger to start a debugging session.
    [debugger.interactive.initializeMessage]
    command = "initialize"
    type = "request"

      [debugger.interactive.initializeMessage.arguments]
      adapterID = "debugpy"
      clientID = "replit"
      clientName = "replit.com"
      columnsStartAt1 = true
      linesStartAt1 = true
      locale = "en-us"
      pathFormat = "path"
      supportsInvalidatedEvent = true
      supportsProgressReporting = true
      supportsRunInTerminalRequest = true
      supportsVariablePaging = true
      supportsVariableType = true

    # How to tell the debugger to start the debuggee application.
    [debugger.interactive.launchMessage]
    command = "attach"
    type = "request"

      [debugger.interactive.launchMessage.arguments]
      logging = {}

# Configures the packager.
[packager]
# Search packages in PyPI.
language = "python3"
# Never attempt to install `unit_tests`. If there are packages that are being
# guessed wrongly, add them here.
ignoredPackages = ["unit_tests"]

  [packager.features]
  enabledForHosting = false
  # Enable searching packages from the sidebar.
  packageSearch = true
  # Enable guessing what packages are needed from the code.
  guessImports = true"#
            .to_string()
            .parse::<Document>()
    }


    macro_rules! add_test {
        ($name:ident, $field:expr, $value:expr, $contents:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut doc = $contents;
                let expected = $expected;
                let field = $field;
                let value = Some($value.to_string());

                let result = handle_add(field.to_string(), value, &mut doc);
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
hi = 123.0
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
foo = [1.0, 2.0, 3.0]
"#
    );

    add_test!(
        simple_push_into_array,
        "arr/2",
        "123",
        r#"arr = [1, 2]"#.parse::<Document>().unwrap(),
        r#"arr = [1, 2, 123.0]"#
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
        r#"meep = "nope""#.parse::<Document>().unwrap(),
        r#"meep = "nope"

[foo]
arr = "yup""#
    );

    add_test!(
        hosting,
        "hosting/route",
        r#""hiiii""#,
        python_dotreplit().unwrap(),
        r#"# The command that runs the program. Commented out because it is not run when the interpreter command is set
# run = ["python3", "main.py"]
# The primary language of the repl. There can be others, though!
language = "python3"
entrypoint = "main.py"
hidden = ["venv", ".config", "**/__pycache__", "**/.mypy_cache", "**/*.pyc"]
audio = true

# Specifies which nix channel to use when building the environment.
[nix]
channel = "stable-21_11"

# Per-language configuration: python3
[languages.python3]
# Treats all files that end with `.py` as Python.
pattern = "**/*.py"
# Tells the workspace editor to syntax-highlight these files as
# Python.
syntax = "python"

  # The command needed to start the Language Server Protocol. For
  # linting and formatting.
  [languages.python3.languageServer]
  start = ["pylsp"]

# The command to start the interpreter.
[interpreter]
  [interpreter.command]
  args = [
    "stderred",
    "--",
    "prybar-python3",
    "-q",
    "--ps1",
    "\u0001\u001b[33m\u0002\u0001\u001b[00m\u0002 ",
    "-i",
  ]
  env = { LD_LIBRARY_PATH = "$PYTHON_LD_LIBRARY_PATH" }

[env]
MPLBACKEND = "TkAgg"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH = "${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY = "https://package-proxy.replit.com/pypi/"
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"

# Enable unit tests. This is only supported for a few languages.
[unitTest]
language = "java"

# Add a debugger!
[debugger]
support = true

  # How to start the debugger.
  [debugger.interactive]
  transport = "localhost:0"
  startCommand = ["dap-python", "main.py"]

    # How to communicate with the debugger.
    [debugger.interactive.integratedAdapter]
    dapTcpAddress = "localhost:0"

    # How to tell the debugger to start a debugging session.
    [debugger.interactive.initializeMessage]
    command = "initialize"
    type = "request"

      [debugger.interactive.initializeMessage.arguments]
      adapterID = "debugpy"
      clientID = "replit"
      clientName = "replit.com"
      columnsStartAt1 = true
      linesStartAt1 = true
      locale = "en-us"
      pathFormat = "path"
      supportsInvalidatedEvent = true
      supportsProgressReporting = true
      supportsRunInTerminalRequest = true
      supportsVariablePaging = true
      supportsVariableType = true

    # How to tell the debugger to start the debuggee application.
    [debugger.interactive.launchMessage]
    command = "attach"
    type = "request"

      [debugger.interactive.launchMessage.arguments]
      logging = {}

# Configures the packager.
[packager]
# Search packages in PyPI.
language = "python3"
# Never attempt to install `unit_tests`. If there are packages that are being
# guessed wrongly, add them here.
ignoredPackages = ["unit_tests"]

  [packager.features]
  enabledForHosting = false
  # Enable searching packages from the sidebar.
  packageSearch = true
  # Enable guessing what packages are needed from the code.
  guessImports = true

[hosting]
route = "hiiii""#
    );
}
