use anyhow::{bail, Context, Result};
use serde_json::{from_str, Value as JValue};
use toml_edit::{Array, ArrayOfTables, Document, InlineTable, Item, Table, Value};

use crate::converter::json_to_toml;
use crate::field_finder::{get_field, DoInsert, TomlValue};

pub fn handle_add(field: &str, value: &str, doc: &mut Document) -> Result<()> {
    let mut path_split = field
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let last_field = path_split.pop().context("Path is empty")?;

    let final_field_value =
        get_field(&path_split, &last_field, DoInsert::Yes, doc).context("Could not find field")?;

    let mut field_value_json: JValue =
        from_str(value).context("parsing value field in add request")?;

    if matches!(field_value_json, JValue::Null) {
        return Ok(());
    }

    filter_nulls(&mut field_value_json);

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

fn filter_nulls(value: &mut JValue) {
    match value {
        JValue::Array(arr) => {
            arr.retain(|v| !matches!(v, JValue::Null));
            arr.iter_mut().for_each(filter_nulls);
        }
        JValue::Object(obj) => {
            obj.retain(|_, v| !matches!(v, JValue::Null));
            obj.values_mut().for_each(filter_nulls);
        }
        _ => return,
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
                pretty_assertions::assert_eq!(doc.to_string().trim(), expected.trim());
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
        r#"arr = [1, 2]"#.parse::<Document>().unwrap(),
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
        r#"meep = "nope""#.parse::<Document>().unwrap(),
        r#"meep = "nope"

[foo]
arr = "yup""#
    );

    add_test!(
        add_array_objects_deep,
        "foo/0/hi",
        r#"123"#,
        r#"top = "hi""#.parse::<Document>().unwrap(),
        r#"top = "hi"

[[foo]]
hi = 123
"#
    );

    add_test!(
        add_array_objects,
        "foo/0",
        r#"{"hi": 123}"#,
        r#"top = "hi""#.parse::<Document>().unwrap(),
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
            .parse::<Document>()
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
        .parse::<Document>()
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
        .parse::<Document>()
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
        .parse::<Document>()
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
        .parse::<Document>()
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

    add_test!(
        add_null_value_to_map,
        "foo/bar",
        "null",
        r#"
[foo]
baz = true
"#
        .parse::<Document>()
        .unwrap(),
        r#"
[foo]
baz = true
"#
    );

    add_test!(
        add_null_value_to_array,
        "foo",
        "[null]",
        r#"
foo = ["hello", "world!"]
"#
        .parse::<Document>()
        .unwrap(),
        r#"
"#
    );

    add_test!(
        append_to_map_with_null_value,
        "interpreter",
        r#"{
    "command": [
        "stderred",
        "--",
        "prybar-python3",
        "-q",
        "--ps1",
        "\u0001\u001b[33m\u0002\u0001\u001b[00m\u0002 ",
        "-i"
    ],
    "prompt": null
}"#,
        r#"
[intepreter.command]
args = []
env = {}
"#
        .parse::<Document>()
        .unwrap(),
        r#"
[interpreter]
command = [
    "stderred",
    "--",
    "prybar-python3",
    "-q",
    "--ps1",
    "\u0001\u001b[33m\u0002\u0001\u001b[00m\u0002 ",
    "-i",
]
"#
    );

    add_test!(
        merge_with_different_structures,
        "interpreter",
        r#"{
    "command": [
        "stderred",
        "--",
        "prybar-python3",
        "-q",
        "--ps1",
        "\u0001\u001b[33m\u0002\u0001\u001b[00m\u0002 ",
        "-i"
    ],
    "prompt": null
}"#,
        r#"
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
"#
        .parse::<Document>()
        .unwrap(),
        r#"
[interpreter]
command = ["stderred", "--", "prybar-python3", "-q", "--ps1", "\u0001\u001B[33m\u0002\u0001\u001B[00m\u0002 ", "-i"]"#
    );

    add_test!(
        merge_with_different_structures_2,
        "interpreter",
        r#"{
    "command": "prybar-python3 -q --ps1 \\u0001\\u001b[33m\\u0002\\u0001\\u001b[00m\\u0002  -i"
}"#,
        r#"
# The command that runs the program. Commented out because it is not run when the interpreter command is set
# run = ["python3", "main.py"]
# The primary language of the repl. There can be others, though!
language = "python3"
# The main file, which will be shown by default in the editor.
entrypoint = "main.py"
# A list of globs that specify which files and directories should
# be hidden in the workspace.
hidden = ["venv", ".config", "**/__pycache__", "**/.mypy_cache", "**/*.pyc"]

# Specifies which nix channel to use when building the environment.
[nix]
channel = "stable-21_11"

# Per-language configuration: python3
[languages.python3]
pattern = "**/*.py"
# Tells the workspace editor to syntax-highlight these files as
# Python.
syntax = "python"

    # The command needed to start the Language Server Protocol. For
    # linting and formatting.
[languages.python3.languageServer]
start = "pylsp"

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
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"

# Enable unit tests. This is only supported for a few languages.
[unitTest]
language = "python3"

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

# These are the files that need to be preserved when this
# language template is used as the base language template
# for Python repos imported from GitHub
[gitHubImport]
requiredFiles = [".replit", "replit.nix", ".config", "venv"]
"#
        .parse::<Document>()
        .unwrap(),
        r#"
# The command that runs the program. Commented out because it is not run when the interpreter command is set
# run = ["python3", "main.py"]
# The primary language of the repl. There can be others, though!
language = "python3"
# The main file, which will be shown by default in the editor.
entrypoint = "main.py"
# A list of globs that specify which files and directories should
# be hidden in the workspace.
hidden = ["venv", ".config", "**/__pycache__", "**/.mypy_cache", "**/*.pyc"]

# Specifies which nix channel to use when building the environment.
[nix]
channel = "stable-21_11"

# Per-language configuration: python3
[languages.python3]
pattern = "**/*.py"
# Tells the workspace editor to syntax-highlight these files as
# Python.
syntax = "python"

    # The command needed to start the Language Server Protocol. For
    # linting and formatting.
[languages.python3.languageServer]
start = "pylsp"

[interpreter]
command = 'prybar-python3 -q --ps1 \u0001\u001b[33m\u0002\u0001\u001b[00m\u0002  -i'

[env]
VIRTUAL_ENV = "/home/runner/${REPL_SLUG}/venv"
PATH = "${VIRTUAL_ENV}/bin"
PYTHONPATH="${VIRTUAL_ENV}/lib/python3.8/site-packages"
REPLIT_POETRY_PYPI_REPOSITORY="https://package-proxy.replit.com/pypi/"
MPLBACKEND="TkAgg"
POETRY_CACHE_DIR="${HOME}/${REPL_SLUG}/.cache/pypoetry"

# Enable unit tests. This is only supported for a few languages.
[unitTest]
language = "python3"

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

# These are the files that need to be preserved when this
# language template is used as the base language template
# for Python repos imported from GitHub
[gitHubImport]
requiredFiles = [".replit", "replit.nix", ".config", "venv"]
"#
    );
}
