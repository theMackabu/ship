use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        tovec => list(..Any),
        tovec => tuple(..Any),
        tostring => string(Any),
        tonumber => string(Any),
        toset => set(Array)
    });
}

fn tovec(args: FuncArgs) -> Result<hcl::Value, String> { Ok(hcl::Value::Array(args.to_vec())) }

fn tostring(args: FuncArgs) -> Result<hcl::Value, String> { Ok(hcl::Value::String(args[0].to_string())) }

fn tonumber(args: FuncArgs) -> Result<hcl::Value, String> {
    let value = args[0].to_string();
    match value.parse::<f64>() {
        Ok(n) => Ok(hcl::Value::Number(hcl::Number::from_f64(n).unwrap())),
        Err(e) => Err(format!("Failed to convert to number: {}", e)),
    }
}

fn toset(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<hcl::Value> = arr.iter().filter(|v| seen.insert(v.to_string())).cloned().collect();
        Ok(hcl::Value::Array(unique))
    } else {
        Err("toset() requires array argument".to_string())
    }
}
