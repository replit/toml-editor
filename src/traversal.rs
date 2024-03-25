use anyhow::{anyhow, Result};
use serde_json::Value as Json;
use toml_edit::{DocumentMut, Item};

pub enum TraverseOps {
    Get(Box<dyn FnOnce(serde_json::Value) -> ()>),
}

pub fn traverse(field: &str, doc: &mut DocumentMut, op: TraverseOps) -> Result<Json> {
    let path_split = field
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let root_key = path_split.get(0).ok_or(anyhow!("Invalid query path!"))?;
    let table = doc.as_table_mut();
    let item = table
        .get_mut(root_key)
        .ok_or(anyhow!("Missing table for traversal"))?;
    do_traverse(path_split, item, op)
}

fn do_traverse(mut path: Vec<String>, item: &mut Item, op: TraverseOps) -> Result<Json> {
    Ok(Json::Null)
}
