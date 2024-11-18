mod cidr;
mod convert;
mod crypto;
mod date;

use hcl::eval::FuncArgs;
use std::fs::{self, File};
use std::io::Read;
use std::str::FromStr;

use md5::{Digest, Md5};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use uuid::Uuid;

use bcrypt::{hash, DEFAULT_COST};

use hcl::eval::Context;
use std::{cell::RefCell, rc::Rc};

pub type Functions<'c> = Rc<RefCell<Context<'c>>>;

pub fn init<'c>() -> Functions<'c> {
    let ctx = Rc::new(RefCell::new(Context::new()));

    cidr::init(ctx.borrow_mut());
    convert::init(ctx.borrow_mut());
    crypto::init(ctx.borrow_mut());
    date::init(ctx.borrow_mut());

    return ctx;
}

fn parse_headers(headers_arg: &Option<&hcl::Value>) -> Option<HeaderMap> {
    match headers_arg {
        Some(headers_value) => {
            if let Some(headers_map) = headers_value.as_object() {
                let mut header_map = HeaderMap::new();

                for (key, value) in headers_map {
                    if let (Ok(header_name), Ok(header_value)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(value.as_str().unwrap_or_default())) {
                        header_map.insert(header_name, header_value);
                    }
                }

                Some(header_map)
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn concat(args: FuncArgs) -> Result<hcl::Value, String> {
    let concatenated = args.iter().map(|arg| arg.as_str().unwrap()).collect::<Vec<&str>>().join("");

    Ok(hcl::Value::from(concatenated))
}

pub fn vec(args: FuncArgs) -> Result<hcl::Value, String> { Ok(hcl::Value::Array(args.to_vec())) }

pub fn length(args: FuncArgs) -> Result<hcl::Value, String> {
    match &args[0] {
        hcl::Value::Array(arr) => Ok(hcl::Value::Number(arr.len().into())),
        hcl::Value::String(s) => Ok(hcl::Value::Number(s.len().into())),
        hcl::Value::Object(map) => Ok(hcl::Value::Number(map.len().into())),
        _ => Err("length() requires array, string or map argument".to_string()),
    }
}

pub fn compact(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let filtered: Vec<hcl::Value> = arr.iter().filter(|v| !matches!(v, hcl::Value::Null)).cloned().collect();
        Ok(hcl::Value::Array(filtered))
    } else {
        Err("compact() requires array argument".to_string())
    }
}

pub fn type_of(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn merge(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn range(args: FuncArgs) -> Result<hcl::Value, String> {
    let start = args[0].as_number().unwrap().as_i64().unwrap();
    let end = args[1].as_number().unwrap().as_i64().unwrap();

    let range_vec: Vec<hcl::Value> = (start..end).map(|n| hcl::Value::Number(n.into())).collect();

    Ok(hcl::Value::Array(range_vec))
}

pub fn reverse(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn sum(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let sum = arr.iter().filter_map(|v| v.as_number()).fold(0.0, |acc, x| acc + x.as_f64().unwrap());
        Ok(hcl::Value::Number(hcl::Number::from_f64(sum).unwrap()))
    } else {
        Err("sum() requires array argument".to_string())
    }
}

pub fn unique(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Array(arr) = &args[0] {
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<hcl::Value> = arr.iter().filter(|v| seen.insert(v.to_string())).cloned().collect();
        Ok(hcl::Value::Array(unique))
    } else {
        Err("unique() requires array argument".to_string())
    }
}

pub fn contains(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn keys(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Object(map) = &args[0] {
        let keys: Vec<hcl::Value> = map.keys().map(|k| hcl::Value::String(k.clone())).collect();
        Ok(hcl::Value::Array(keys))
    } else {
        Err("keys() requires map argument".to_string())
    }
}

pub fn values(args: FuncArgs) -> Result<hcl::Value, String> {
    if let hcl::Value::Object(map) = &args[0] {
        let values: Vec<hcl::Value> = map.values().cloned().collect();
        Ok(hcl::Value::Array(values))
    } else {
        Err("values() requires map argument".to_string())
    }
}

pub fn split(args: FuncArgs) -> Result<hcl::Value, String> {
    if let (hcl::Value::String(s), hcl::Value::String(delimiter)) = (&args[0], &args[1]) {
        let parts: Vec<hcl::Value> = s.split(delimiter).map(|part| hcl::Value::String(part.to_string())).collect();
        Ok(hcl::Value::Array(parts))
    } else {
        Err("split() requires string and delimiter string arguments".to_string())
    }
}

pub fn join(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn max(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn min(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn flatten(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn file(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    match fs::read_to_string(path) {
        Ok(contents) => Ok(hcl::Value::String(contents)),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

pub fn http_get(args: FuncArgs) -> Result<hcl::Value, String> {
    let url = args[0].as_str().unwrap();
    let headers = parse_headers(&args.get(1));

    let client = reqwest::blocking::Client::new();
    let mut request = client.get(url);

    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    match request.send() {
        Ok(response) => match response.text() {
            Ok(text) => Ok(hcl::Value::String(text)),
            Err(e) => Err(format!("Failed to read response: {}", e)),
        },
        Err(e) => Err(format!("HTTP GET request failed: {}", e)),
    }
}

pub fn vault_kv(args: FuncArgs) -> Result<hcl::Value, String> {
    let config = crate::config::read();
    let value = args[0].as_str().unwrap();

    let mut key = None;

    if args.len() > 2 {
        return Err("Too many arguments, expected at most 2".into());
    }

    if args.len() > 1 && args[1] != hcl::Value::Null {
        key = Some(args[1].to_owned());
    }

    let client = reqwest::blocking::Client::new();
    let request = client
        .get(format!("{}/v1/kv/data/{value}", config.settings.vault_url))
        .header("X-Vault-Token", config.settings.vault_token);

    match request.send() {
        Ok(response) => match response.json::<hcl::Object<String, hcl::Value>>() {
            Ok(json) => match json.get("data") {
                Some(data) => {
                    let values = match data.as_object() {
                        Some(values) => values.get("data"),
                        None => return Ok(data.to_owned()),
                    };

                    let secret_map = match values {
                        Some(secret) => secret.as_object(),
                        None => return Ok(data.to_owned()),
                    };

                    let key_value = match key {
                        Some(key) => key,
                        None => return Ok(hcl::Value::Object(secret_map.expect("Expected valid early returns").to_owned())),
                    };

                    let key = match key_value.as_str() {
                        Some(key) => key,
                        None => return Ok(hcl::Value::Object(secret_map.expect("Expected valid early returns").to_owned())),
                    };

                    let secret = match secret_map {
                        Some(secret) => secret.get(key),
                        None => return Ok(hcl::Value::Object(secret_map.expect("Expected valid early returns").to_owned())),
                    };

                    if let Some(val) = secret {
                        return Ok(val.to_owned());
                    }

                    Ok(data.to_owned())
                }
                None => Err("Unable to decode json".to_string()),
            },
            Err(e) => Err(format!("Failed to read response: {}", e)),
        },
        Err(e) => Err(format!("HTTP GET request failed: {}", e)),
    }
}

pub fn http_post(args: FuncArgs) -> Result<hcl::Value, String> {
    let url = args[0].as_str().unwrap();
    let body = args[1].as_str().unwrap();
    let headers = parse_headers(&args.get(2));

    let client = reqwest::blocking::Client::new();
    let mut request = client.post(url).body(body.to_string());

    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    match request.send() {
        Ok(response) => match response.text() {
            Ok(text) => Ok(hcl::Value::String(text)),
            Err(e) => Err(format!("Failed to read response: {}", e)),
        },
        Err(e) => Err(format!("HTTP POST request failed: {}", e)),
    }
}

pub fn http_post_json(args: FuncArgs) -> Result<hcl::Value, String> {
    let url = args[0].as_str().unwrap();
    let json_body = args[1].to_string();
    let headers = parse_headers(&args.get(2));

    let client = reqwest::blocking::Client::new();
    let mut request = client.post(url).header("Content-Type", "application/json").body(json_body);

    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    match request.send() {
        Ok(response) => match response.text() {
            Ok(text) => Ok(hcl::Value::String(text)),
            Err(e) => Err(format!("Failed to read response: {}", e)),
        },
        Err(e) => Err(format!("HTTP POST request failed: {}", e)),
    }
}

pub fn http_put(args: FuncArgs) -> Result<hcl::Value, String> {
    let url = args[0].as_str().unwrap();
    let body = args[1].as_str().unwrap();
    let headers = parse_headers(&args.get(2));

    let client = reqwest::blocking::Client::new();
    let mut request = client.put(url).body(body.to_string());

    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    match request.send() {
        Ok(response) => match response.text() {
            Ok(text) => Ok(hcl::Value::String(text)),
            Err(e) => Err(format!("Failed to read response: {}", e)),
        },
        Err(e) => Err(format!("HTTP PUT request failed: {}", e)),
    }
}

pub fn format(args: FuncArgs) -> Result<hcl::Value, String> {
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

pub fn upper(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.to_uppercase()))
}

pub fn lower(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.to_lowercase()))
}

pub fn trimspace(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    Ok(hcl::Value::String(input.trim().to_string()))
}

pub fn trim(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let cutset = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.trim_matches(|c| cutset.contains(c)).to_string()))
}

pub fn trimprefix(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let prefix = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.strip_prefix(prefix).unwrap_or(input).to_string()))
}

pub fn trimsuffix(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let suffix = args[1].as_str().unwrap();
    Ok(hcl::Value::String(input.strip_suffix(suffix).unwrap_or(input).to_string()))
}

pub fn abs(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().abs()).unwrap()))
    } else {
        Err("abs() requires number argument".to_string())
    }
}

pub fn ceil(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().ceil()).unwrap()))
    } else {
        Err("ceil() requires number argument".to_string())
    }
}

pub fn floor(args: FuncArgs) -> Result<hcl::Value, String> {
    if let Some(num) = args[0].as_number() {
        Ok(hcl::Value::Number(hcl::Number::from_f64(num.as_f64().unwrap().floor()).unwrap()))
    } else {
        Err("floor() requires number argument".to_string())
    }
}

pub fn bcrypt_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    match hash(input.as_bytes(), DEFAULT_COST) {
        Ok(hashed) => Ok(hcl::Value::String(hashed)),
        Err(e) => Err(format!("Bcrypt error: {}", e)),
    }
}

pub fn filemd5(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn filesha1(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut hasher = Sha1::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn filesha256(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn filesha512(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut hasher = Sha512::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn md5_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn sha1_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn sha256_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn sha512_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha512::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

pub fn uuid_gen(_args: FuncArgs) -> Result<hcl::Value, String> { Ok(hcl::Value::String(Uuid::new_v4().to_string())) }

pub fn uuidv5(args: FuncArgs) -> Result<hcl::Value, String> {
    let namespace = args[0].as_str().unwrap();
    let name = args[1].as_str().unwrap();

    let namespace_uuid = Uuid::parse_str(namespace).map_err(|e| format!("Invalid namespace UUID: {}", e))?;

    let uuid = Uuid::new_v5(&namespace_uuid, name.as_bytes());

    Ok(hcl::Value::String(uuid.to_string()))
}

pub fn parseint(args: FuncArgs) -> Result<hcl::Value, String> {
    let value = args[0].as_str().unwrap();

    match i64::from_str(value) {
        Ok(n) => Ok(hcl::Value::Number(hcl::Number::from_f64(n as f64).unwrap())),
        Err(e) => Err(format!("Failed to parse integer: {}", e)),
    }
}
