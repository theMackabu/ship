use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

use base64::{engine::general_purpose::STANDARD as base64_engine, Engine};
use urlencoding::{decode as url_decode, encode as url_encode};

use serde_json::{from_str as from_json_str, to_string as to_json_string, Value as JsonValue};
use serde_yaml_ng::{from_str as from_yaml_str, to_string as to_yaml_string};

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        base64encode => encode::base64(String),
        base64decode => decode::base64(String),
        jsonencode => encode::json(Any),
        jsondecode => decode::json(String),
        urlencode => encode::url(String),
        urldecode => decode::url(String),
        yamlencode => encode::yaml(Any),
        yamldecode => decode::yaml(String)
    });
}

fn base64encode(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(base64_engine.encode(input.as_bytes())))
}

fn base64decode(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    match base64_engine.decode(input) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(string) => Ok(hcl::Value::String(string)),
            Err(e) => Err(format!("Invalid UTF-8 in decoded base64: {}", e)),
        },
        Err(e) => Err(format!("Invalid base64: {}", e)),
    }
}

fn jsonencode(args: FuncArgs) -> Result<hcl::Value, String> {
    let json_value = hcl_to_json(&args[0]);
    match to_json_string(&json_value) {
        Ok(json_str) => Ok(hcl::Value::String(json_str)),
        Err(e) => Err(format!("JSON encoding error: {}", e)),
    }
}

fn jsondecode(args: FuncArgs) -> Result<hcl::Value, String> {
    let json_str = args[0].as_str().unwrap();
    match from_json_str(json_str) {
        Ok(json_value) => Ok(json_to_hcl(json_value)),
        Err(e) => Err(format!("JSON decoding error: {}", e)),
    }
}

fn urlencode(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(url_encode(input).to_string()))
}

fn urldecode(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    match url_decode(input) {
        Ok(decoded) => Ok(hcl::Value::String(decoded.into_owned())),
        Err(e) => Err(format!("URL decoding error: {}", e)),
    }
}

fn yamlencode(args: FuncArgs) -> Result<hcl::Value, String> {
    let json_value = hcl_to_json(&args[0]);
    match to_yaml_string(&json_value) {
        Ok(yaml_str) => Ok(hcl::Value::String(yaml_str)),
        Err(e) => Err(format!("YAML encoding error: {}", e)),
    }
}

fn yamldecode(args: FuncArgs) -> Result<hcl::Value, String> {
    let yaml_str = args[0].as_str().unwrap();
    match from_yaml_str(yaml_str) {
        Ok(json_value) => Ok(json_to_hcl(json_value)),
        Err(e) => Err(format!("YAML decoding error: {}", e)),
    }
}

fn hcl_to_json(value: &hcl::Value) -> JsonValue {
    match value {
        hcl::Value::Null => JsonValue::Null,
        hcl::Value::Bool(b) => JsonValue::Bool(*b),
        hcl::Value::Number(n) => JsonValue::Number(serde_json::Number::from_f64(n.as_f64().unwrap()).unwrap()),
        hcl::Value::String(s) => JsonValue::String(s.clone()),
        hcl::Value::Array(arr) => JsonValue::Array(arr.iter().map(hcl_to_json).collect()),
        hcl::Value::Object(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                json_map.insert(k.clone(), hcl_to_json(v));
            }
            JsonValue::Object(json_map)
        }
    }
}

fn json_to_hcl(value: JsonValue) -> hcl::Value {
    match value {
        JsonValue::Null => hcl::Value::Null,
        JsonValue::Bool(b) => hcl::Value::Bool(b),
        JsonValue::Number(n) => hcl::Value::Number(hcl::Number::from_f64(n.as_f64().unwrap()).unwrap()),
        JsonValue::String(s) => hcl::Value::String(s),
        JsonValue::Array(arr) => hcl::Value::Array(arr.into_iter().map(json_to_hcl).collect()),
        JsonValue::Object(map) => {
            let mut hcl_map = hcl::Map::new();
            for (k, v) in map {
                hcl_map.insert(k, json_to_hcl(v));
            }
            hcl::Value::Object(hcl_map)
        }
    }
}
