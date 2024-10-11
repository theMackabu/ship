mod config;
mod models;

use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, str::FromStr};

use hcl::eval::{Context, Evaluate};
use hcl::Template;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

use tide::{convert::json, utils::After, Error, Request, Response};
use tide_tracing::TraceMiddleware;

#[derive(Deserialize)]
struct Params {
    lang: Option<String>,
}

pub enum Language {
    YAML,
    JSON,
    TOML,
    None,
}

impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "toml" => Language::TOML,
            "json" => Language::JSON,
            "yml" | "yaml" => Language::YAML,
            _ => Language::None,
        })
    }
}

impl Language {
    fn parse(s: &str) -> Language { Language::from_str(s).expect("expected valid enum item") }
}

pub struct HclConverter<'c> {
    data: String,
    language: Option<String>,
    ctx: Rc<RefCell<Context<'c>>>,
}

impl<'c> HclConverter<'c> {
    pub fn new(input: &str) -> Result<Self, Error> {
        let default = Self {
            language: None,
            data: input.to_owned(),
            ctx: Rc::new(RefCell::new(Context::new())),
        };

        Ok(default)
    }

    pub fn read<F>(path: F) -> Result<Self, Error>
    where
        F: Into<PathBuf>,
    {
        let content = fs::read_to_string(path.into())?;
        Self::new(&content)
    }

    pub fn declare<I, T>(&mut self, name: I, value: T)
    where
        I: Into<hcl::Identifier>,
        T: Into<hcl::Value>,
    {
        self.ctx.borrow_mut().declare_var(name.into(), value.into());
    }

    pub fn fetch_meta(&mut self) -> Result<(), Error> {
        if let Some(obj) = self.eval()?.as_object() {
            self.language = obj.get("meta_language").and_then(|m| if let hcl::Value::String(s) = m { Some(s.clone()) } else { None });
        }
        Ok(())
    }

    pub fn toml(&self) -> Result<String, Error> {
        let value = self.to_toml(&self.result()?);
        Ok(toml::to_string_pretty(&value)?)
    }

    pub fn yaml(&self) -> Result<String, Error> {
        let value = self.to_yaml(&self.result()?);
        Ok(serde_yaml_ng::to_string(&value)?)
    }

    pub fn json(&self) -> Result<String, Error> {
        let value = self.to_json(&self.result()?);
        Ok(serde_json::to_string_pretty(&value)?)
    }

    fn eval(&self) -> Result<hcl::Value, Error> {
        let tmpl = Template::from_str(&self.data)?;
        let data = tmpl.evaluate(&self.ctx.borrow())?;

        Ok(hcl::from_str(&data)?)
    }

    fn result(&self) -> Result<hcl::Value, Error> {
        let mut value = self.eval()?;

        if let hcl::Value::Object(obj) = &mut value {
            obj.shift_remove("meta_language");
        }

        Ok(value)
    }

    fn to_toml(&self, hcl: &hcl::Value) -> TomlValue {
        match hcl {
            hcl::Value::Null => TomlValue::String("null".to_string()), // TOML doesn't have a native null
            hcl::Value::String(s) => TomlValue::String(s.clone()),
            hcl::Value::Number(n) => {
                if n.is_i64() {
                    TomlValue::Integer(n.as_i64().unwrap())
                } else {
                    TomlValue::Float(n.as_f64().unwrap())
                }
            }
            hcl::Value::Bool(b) => TomlValue::Boolean(*b),
            hcl::Value::Array(arr) => TomlValue::Array(arr.iter().map(|v| self.to_toml(v)).collect()),
            hcl::Value::Object(obj) => {
                let mut map = toml::map::Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), self.to_toml(v));
                }
                TomlValue::Table(map)
            }
        }
    }

    fn to_yaml(&self, hcl: &hcl::Value) -> YamlValue {
        match hcl {
            hcl::Value::Null => YamlValue::Null,
            hcl::Value::String(s) => YamlValue::String(s.clone()),
            hcl::Value::Number(n) => {
                if n.is_i64() {
                    YamlValue::Number(n.as_i64().unwrap().into())
                } else {
                    YamlValue::Number(n.as_f64().unwrap().into())
                }
            }
            hcl::Value::Bool(b) => YamlValue::Bool(*b),
            hcl::Value::Array(arr) => YamlValue::Sequence(arr.iter().map(|v| self.to_yaml(v)).collect()),
            hcl::Value::Object(obj) => {
                let mut map = serde_yaml_ng::Mapping::new();
                for (k, v) in obj {
                    map.insert(YamlValue::String(k.clone()), self.to_yaml(v));
                }
                YamlValue::Mapping(map)
            }
        }
    }

    fn to_json(&self, hcl: &hcl::Value) -> JsonValue {
        match hcl {
            hcl::Value::Null => JsonValue::Null,
            hcl::Value::String(s) => JsonValue::String(s.clone()),
            hcl::Value::Number(n) => {
                if n.is_i64() {
                    JsonValue::Number(n.as_i64().unwrap().into())
                } else {
                    JsonValue::Number(serde_json::Number::from_f64(n.as_f64().unwrap()).unwrap_or(serde_json::Number::from(0)))
                }
            }
            hcl::Value::Bool(b) => JsonValue::Bool(*b),
            hcl::Value::Array(arr) => JsonValue::Array(arr.iter().map(|v| self.to_json(v)).collect()),
            hcl::Value::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), self.to_json(v));
                }
                JsonValue::Object(map)
            }
        }
    }
}

async fn test(req: Request<models::Config>) -> tide::Result<String> {
    let params: Params = req.query()?;
    let base = &req.state().settings.storage;
    let file = req.param("path").unwrap_or_default();

    let mut hcl = match HclConverter::read(base.join(file)) {
        Ok(converter) => converter,
        Err(_) => HclConverter::read(base.join(file).join("index.hcl"))?,
    };

    hcl.declare("domain", "themackabu.dev");
    hcl.fetch_meta()?;

    let lang = params.lang.unwrap_or(hcl.language.to_owned().unwrap_or_default());

    let data = match Language::parse(&lang) {
        Language::TOML => hcl.toml(),
        Language::JSON => hcl.json(),
        Language::YAML => hcl.yaml(),
        Language::None => return Err(tide::Error::from_str(400, "language not found")),
    };

    Ok(data?)
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let config = config::read();
    let sub = tracing_subscriber::fmt().json();
    let mut app = tide::with_state(config.to_owned());

    sub.with_max_level(tracing::Level::INFO).init();
    app.with(TraceMiddleware::new());

    app.with(After(|mut res: Response| async {
        if let Some(error) = res.take_error() {
            let status = error.status();

            res.set_status(status);
            res.set_body(json!({ "code": status, "error": error.to_string() }));
        }
        Ok(res)
    }));

    app.at("/get/*path").get(test);
    app.listen(config.settings.listen).await?;

    Ok(())
}
