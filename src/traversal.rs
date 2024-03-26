use anyhow::{anyhow, Result};
use serde_json::Map;
use serde_json::Value as Json;
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

#[derive(Debug)]
pub enum At<'a> {
    Array(&'a mut Array),
    ArrayOfTables(&'a mut ArrayOfTables),
    Item(&'a mut Item),
    Table(&'a mut Table),
    Value(&'a mut Value),
}

pub enum TraverseOps {
    Get,
}

/*
array:           Type representing a TOML array, payload of `Value::Array`
array_of_tables: Type representing a TOML array of tables
item:            Type representing either a value, a table, an array of tables, or none.
table:           Type representing a TOML non-inline table
value:           Representation of a TOML Value (as part of a Key/Value Pair).

*/

pub fn traverse<'a>(
    op: TraverseOps,
    doc: &'a mut DocumentMut,
    field: &str,
) -> Result<Option<Json>> {
    let path_split = field.split('/').collect::<Vec<&str>>();
    let path_slice = path_split.as_slice();
    let root_key = path_split.get(0).ok_or(anyhow!("Invalid query path!"))?;
    let table = doc.as_table_mut();
    let item = table
        .get_mut(root_key)
        .ok_or(anyhow!("Missing table for traversal"))?;
    do_traverse(&path_slice[1..], &mut At::Item::<'a>(item), op)
}

fn do_traverse(path: &[&str], item: &mut At, op: TraverseOps) -> Result<Option<Json>> {
    if !path.is_empty() {
        return item.fold(path[0], &path[1..], op);
    }

    match op {
        TraverseOps::Get => item.to_value().map(|v| Some(v)),
    }
}

impl At<'_> {
    fn fold(&mut self, key: &str, path: &[&str], op: TraverseOps) -> Result<Option<Json>> {
        match self {
            At::Array(arr) => {
                let index = key
                    .parse::<usize>()
                    .map_err(|_| anyhow!("Key is not a valid integer"))?;
                match arr.get_mut(index) {
                    Some(v) => do_traverse(path, &mut At::Value(v), op),
                    None => Ok(None),
                }
            }
            At::ArrayOfTables(aar) => {
                let index = key
                    .parse::<usize>()
                    .map_err(|_| anyhow!("Key is not a valid usize"))?;
                let member = aar.get_mut(index).ok_or(anyhow!("Array out of range"))?;
                do_traverse(path, &mut At::Table(member), op)
            }
            At::Item(item) => match item {
                Item::ArrayOfTables(aar) => At::ArrayOfTables(aar).fold(key, path, op),
                Item::Table(table) => At::Table(table).fold(key, path, op),
                Item::Value(value) => At::Value(value).fold(key, path, op),
                _ => Err(anyhow!("Unable to index item {:?} with {:?}", item, key)),
            },
            At::Table(table) => {
                let mut found = table
                    .get_mut(key)
                    .ok_or(anyhow!("Unable to index table with {:?}", key))?;
                do_traverse(path, &mut At::Item(&mut found), op)
            }
            At::Value(value) => match value {
                Value::Array(arr) => At::Array(arr).fold(key, path, op),
                Value::InlineTable(table) => {
                    let mut found = table
                        .get_mut(key)
                        .ok_or(anyhow!("Unable to index table with {:?}", key))?;
                    do_traverse(path, &mut At::Value(&mut found), op)
                }
                _ => Err(anyhow!("Unable to index value {:?} with {:?}", value, key)),
            },
        }
    }

    pub fn to_value(&mut self) -> Result<serde_json::Value> {
        match self {
            At::Array(arr) => {
                let xs = arr
                    .iter_mut()
                    .map(|val| At::Value(val).to_value())
                    .collect::<Result<Vec<Json>>>()?;
                Ok(Json::Array(xs))
            }
            At::ArrayOfTables(aar) => {
                let result = aar
                    .iter_mut()
                    .map(|table| At::Table(table).to_value())
                    .collect::<Result<Vec<Json>>>()?;
                Ok(Json::Array(result))
            }
            At::Item(item) => match item {
                Item::None => Ok(Json::Null),
                Item::Value(value) => At::Value(value).to_value(),
                Item::ArrayOfTables(aar) => At::ArrayOfTables(aar).to_value(),
                Item::Table(table) => At::Table(table).to_value(),
            },
            At::Value(value) => match value {
                Value::String(s) => {
                    s.fmt();
                    Ok(Json::String(s.value().clone()))
                }
                Value::Integer(i) => Ok(Json::Number(serde_json::Number::from(
                    i.clone().into_value(),
                ))),
                Value::Float(f) => {
                    let n = serde_json::Number::from_f64(f.clone().into_value()).ok_or(anyhow!(
                        "Unable to parse float as JSON: infinite and NaN are not allowed"
                    ))?;
                    Ok(Json::Number(n))
                }
                Value::Boolean(b) => Ok(Json::Bool(b.clone().into_value())),
                Value::Array(arr) => At::Array(arr).to_value(),
                Value::Datetime(dt) => Ok(Json::String(dt.to_string())),
                Value::InlineTable(table) => {
                    let inner: Map<String, Json> = table
                        .iter_mut()
                        .map(|(k, v)| At::Value(v).to_value().map(|v| (k.to_string(), v)))
                        .collect::<Result<Map<String, Json>>>()?;
                    Ok(Json::Object(inner))
                }
            },
            At::Table(table) => {
                let inner: Map<String, Json> = table
                    .iter_mut()
                    .map(|(k, i)| At::Item(i).to_value().map(|v| (k.to_string(), v)))
                    .collect::<Result<Map<String, Json>>>()?;
                Ok(Json::Object(inner))
            }
        }
    }
}
