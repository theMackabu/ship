use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::{cell::RefMut, str::FromStr};

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        abs => abs(Number),
        ceil => ceil(Number),
        floor => floor(Number),
        max => max(Array),
        min => min(Array),
        sum => sum(Array),
        parseint => parseint(String)
    });
}

fn sum(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let sum = arr.iter().filter_map(|v| v.as_number()).fold(0.0, |acc, x| acc + x.as_f64().unwrap());
        Ok(hcl::Value::Number(hcl::Number::from_f64(sum).unwrap()))
    } else {
        Err("sum() requires array argument".to_string())
    }
}

fn max(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        arr.iter()
            .filter_map(|v| v.as_number())
            .map(|n| n.as_f64().unwrap())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .map(|n| hcl::Value::Number(hcl::Number::from_f64(n).unwrap()))
            .ok_or_else(|| "max() requires non-empty array of numbers".to_string())
    } else {
        Err("max() requires array argument".to_string())
    }
}

fn min(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        arr.iter()
            .filter_map(|v| v.as_number())
            .map(|n| n.as_f64().unwrap())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .map(|n| hcl::Value::Number(hcl::Number::from_f64(n).unwrap()))
            .ok_or_else(|| "min() requires non-empty array of numbers".to_string())
    } else {
        Err("min() requires array argument".to_string())
    }
}

fn abs(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().abs()).unwrap()))
    } else {
        Err("abs() requires number argument".to_string())
    }
}

fn ceil(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().ceil()).unwrap()))
    } else {
        Err("ceil() requires number argument".to_string())
    }
}

fn floor(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().floor()).unwrap()))
    } else {
        Err("floor() requires number argument".to_string())
    }
}

fn parseint(args: FuncArgs) -> Result<hcl::Value, String> {
    let value = args[0].as_str().unwrap();

    match i64::from_str(value) {
        Ok(n) => Ok(hcl::Value::Number(hcl::Number::from_f64(n as f64).unwrap())),
        Err(e) => Err(format!("Failed to parse integer: {}", e)),
    }
}
