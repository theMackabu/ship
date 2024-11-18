use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        join => join(Array, String),
        split => split(String, String),
        range => range(Number, Number),
        merge => merge(..Object),
        format => format(..Any),
        concat => concat(..String),
        length => length(Any),
        unique => unique(Array),
        compact => compact(Object),
        type_of => type_of(Any),
        reverse => reverse(Any),
        flatten => flatten(Array),
        contains => contains(Any, Any)
    });
}

fn join(args: FuncArgs) -> Result<hcl::Value, String> {
    if let (hcl::Value::Array(arr), hcl::Value::String(separator)) = (&args[0], &args[1]) {
        let strings: Result<Vec<String>, String> = arr
            .iter()
            .map(|v| match v {
                hcl::Value::String(s) => Ok(s.clone()),
                _ => Ok(v.to_string()),
            })
            .collect();
        Ok(hcl::Value::String(strings?.join(separator)))
    } else {
        Err("join() requires array and separator string arguments".to_string())
    }
}

fn split(args: FuncArgs) -> Result<hcl::Value, String> {
    if let (hcl::Value::String(s), hcl::Value::String(delimiter)) = (&args[0], &args[1]) {
        let parts: Vec<hcl::Value> = s.split(delimiter).map(|part| hcl::Value::String(part.to_string())).collect();
        Ok(hcl::Value::Array(parts))
    } else {
        Err("split() requires string and delimiter string arguments".to_string())
    }
}

fn range(args: FuncArgs) -> Result<hcl::Value, String> {
    let start = args[0].as_number().unwrap().as_i64().unwrap();
    let end = args[1].as_number().unwrap().as_i64().unwrap();

    let range_vec: Vec<hcl::Value> = (start..end).map(|n| hcl::Value::Number(n.into())).collect();

    Ok(hcl::Value::Array(range_vec))
}

fn type_of(args: FuncArgs) -> Result<hcl::Value, String> {
    let type_name = match &args[0] {
        hcl::Value::Null => "null",
        hcl::Value::Bool(_) => "boolean",
        hcl::Value::Number(_) => "number",
        hcl::Value::String(_) => "string",
        hcl::Value::Array(_) => "array",
        hcl::Value::Object(_) => "object",
    };
    Ok(hcl::Value::String(type_name.to_string()))
}

fn merge(args: FuncArgs) -> Result<hcl::Value, String> {
    let mut result = hcl::Map::new();
    for arg in args.iter() {
        if let hcl::Value::Object(map) = arg {
            result.extend(map.clone());
        } else {
            return Err("merge() requires map arguments".to_string());
        }
    }
    Ok(hcl::Value::Object(result))
}

fn format(args: FuncArgs) -> Result<hcl::Value, String> {
    if args.is_empty() {
        return Err("format() requires at least one argument".to_string());
    }

    let format_str = args[0].as_str().unwrap();
    let format_args = &args[1..];

    let mut result = format_str.to_string();
    let mut arg_index = 0;

    while let Some(start) = result.find('%') {
        if start + 1 >= result.len() {
            return Err("Invalid format string: % at end of string".to_string());
        }

        if arg_index >= format_args.len() {
            return Err("Not enough arguments for format string".to_string());
        }

        let format_type = result.chars().nth(start + 1).unwrap();
        let replacement = match format_type {
            's' => format_args[arg_index].to_string(),
            'd' => match format_args[arg_index].as_number() {
                Some(n) => format!("{}", n.as_f64().unwrap() as i64),
                None => return Err("Expected number for %d format".to_string()),
            },
            'f' => match format_args[arg_index].as_number() {
                Some(n) => format!("{}", n.as_f64().unwrap()),
                None => return Err("Expected number for %f format".to_string()),
            },
            '%' => {
                arg_index -= 1;
                "%".to_string()
            }
            _ => return Err(format!("Unknown format specifier %{}", format_type)),
        };

        result.replace_range(start..start + 2, &replacement);
        arg_index += 1;
    }

    Ok(hcl::Value::String(result))
}

fn concat(args: FuncArgs) -> Result<hcl::Value, String> {
    let concatenated = args.iter().map(|arg| arg.as_str().unwrap()).collect::<Vec<&str>>().join("");

    Ok(hcl::Value::from(concatenated))
}

fn length(args: FuncArgs) -> Result<hcl::Value, String> {
    match &args[0] {
        hcl::Value::Array(arr) => Ok(hcl::Value::Number(arr.len().into())),
        hcl::Value::String(s) => Ok(hcl::Value::Number(s.len().into())),
        hcl::Value::Object(map) => Ok(hcl::Value::Number(map.len().into())),
        _ => Err("length() requires array, string or map argument".to_string()),
    }
}

fn compact(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let filtered: Vec<hcl::Value> = arr.iter().filter(|v| !matches!(v, hcl::Value::Null)).cloned().collect();
        Ok(hcl::Value::Array(filtered))
    } else {
        Err("compact() requires array argument".to_string())
    }
}

fn reverse(args: FuncArgs) -> Result<hcl::Value, String> {
    match &args[0] {
        hcl::Value::Array(arr) => {
            let mut reversed = arr.clone();
            reversed.reverse();
            Ok(hcl::Value::Array(reversed))
        }
        hcl::Value::String(s) => Ok(hcl::Value::String(s.chars().rev().collect())),
        _ => Err("reverse() requires array or string argument".to_string()),
    }
}

fn unique(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<hcl::Value> = arr.iter().filter(|v| seen.insert(v.to_string())).cloned().collect();
        Ok(hcl::Value::Array(unique))
    } else {
        Err("unique() requires array argument".to_string())
    }
}

fn contains(args: FuncArgs) -> Result<hcl::Value, String> {
    match &args[0] {
        hcl::Value::Array(arr) => Ok(hcl::Value::Bool(arr.contains(&args[1]))),
        hcl::Value::String(s) => {
            if let hcl::Value::String(search) = &args[1] {
                Ok(hcl::Value::Bool(s.contains(search)))
            } else {
                Err("Second argument must be string for string contains".to_string())
            }
        }
        _ => Err("contains() requires array or string as first argument".to_string()),
    }
}

fn flatten(args: FuncArgs) -> Result<hcl::Value, String> {
    pub fn flatten_inner(arr: &[hcl::Value], result: &mut Vec<hcl::Value>) {
        for value in arr {
            match value {
                hcl::Value::Array(nested) => flatten_inner(nested, result),
                other => result.push(other.clone()),
            }
        }
    }

    if let hcl::Value::Array(arr) = &args[0] {
        let mut flattened = Vec::new();
        flatten_inner(arr, &mut flattened);
        Ok(hcl::Value::Array(flattened))
    } else {
        Err("flatten() requires array argument".to_string())
    }
}
