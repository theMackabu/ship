use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::cell::RefMut;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        vault_kv => secret::kv(String, ..Nullable),
        http_get => http::get(String, ..Any),
        http_post => http::post(String, String, ..Any),
        http_json => http::post_json(String, Any, ..Any),
        http_put => http::put(String, String, ..Any)
    });
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

fn vault_kv(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn http_get(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn http_post(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn http_json(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn http_put(args: FuncArgs) -> Result<hcl::Value, String> {
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
