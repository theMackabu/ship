use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        keys => map::keys(Object),
        values => map::values(Object),
        upper => str::upper(String),
        lower => str::lower(String),
        trim => str::trim(String, String),
        trimspace => str::trimspace(String),
        trimprefix => str::trimprefix(String, String),
        trimsuffix => str::trimsuffix(String, String)
    });
}

fn keys(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Object(map) = &args[0] {
        let keys: Vec<hcl::Value> = map.keys().map(|k| hcl::Value::String(k.clone())).collect();
        Ok(hcl::Value::Array(keys))
    } else {
        Err("keys() requires map argument".to_string())
    }
}

fn values(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Object(map) = &args[0] {
        let values: Vec<hcl::Value> = map.values().cloned().collect();
        Ok(hcl::Value::Array(values))
    } else {
        Err("values() requires map argument".to_string())
    }
}

fn upper(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.to_uppercase()))
}

fn lower(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.to_lowercase()))
}

fn trimspace(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.trim().to_string()))
}

fn trim(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let cutset = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.trim_matches(|c| cutset.contains(c)).to_string()))
}

fn trimprefix(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let prefix = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.strip_prefix(prefix).unwrap_or(input).to_string()))
}

fn trimsuffix(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let suffix = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.strip_suffix(suffix).unwrap_or(input).to_string()))
}
