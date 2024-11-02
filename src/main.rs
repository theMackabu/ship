mod config;
mod models;

use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, str::FromStr};

use hcl::eval::{Context, Evaluate};
use hcl::{Block, Template};

use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

use tide::{utils::After, Error, Request, Response};
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
    file: Option<String>,
    export: Option<String>,
    ctx: Rc<RefCell<Context<'c>>>,
}

impl<'c> HclConverter<'c> {
    pub fn new(input: &str) -> Result<Self, Error> {
        let default = Self {
            file: None,
            export: None,
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

    pub fn fetch_env(&mut self) -> Result<(), Error> {
        let value: hcl::Value = hcl::from_str(&self.data)?;
        let obj = value.as_object().ok_or(Error::from_str(500, "Invalid root object"))?;
        let env = obj.get("env").and_then(|m| m.as_object());

        if let Some(env) = env {
            self.declare("env", env.to_owned());
        }

        Ok(())
    }

    pub fn fetch_meta(&mut self) -> Result<(), Error> {
        let value: hcl::Value = hcl::from_str(&self.data)?;
        let obj = value.as_object().ok_or(Error::from_str(500, "Invalid root object"))?;

        let meta = obj.get("meta").and_then(|m| m.as_object()).ok_or(Error::from_str(404, "Missing meta object"))?;
        let file = meta.get("file").and_then(|m| m.as_str()).map(|s| s.to_string());

        match meta.get("kind").and_then(|k| k.as_str()) {
            Some("docker") => {
                if let Some(services) = obj.get("services").and_then(hcl::Value::as_object) {
                    self.declare("services", services.keys().cloned().collect::<hcl::Value>());
                }
            }
            _ => {}
        }

        for (key, value) in meta {
            self.declare(key.as_str(), value.to_owned());
        }

        if let Some(path) = file {
            let (name, extension) = match path.rsplit_once('.') {
                Some((name, ext)) => (name.to_string(), Some(ext.to_string())),
                None => (path, meta.get("export").and_then(|m| m.as_str()).map(|s| s.to_string())),
            };

            self.file = Some(name);
            self.export = extension;
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
            obj.shift_remove("meta");
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

async fn compile(req: Request<models::Config>) -> tide::Result {
    let mut res = Response::new(200);

    let params: Params = req.query()?;
    let base = &req.state().settings.storage;
    let file = req.param("path").unwrap_or_default();

    let mut hcl = match HclConverter::read(base.join(file)) {
        Ok(converter) => converter,
        Err(_) => HclConverter::read(base.join(file).join("index.hcl"))?,
    };

    let version = Block::builder("version").add_attribute(("syntax", "v1")).add_attribute(("pkg", env!("CARGO_PKG_VERSION"))).build();

    hcl.fetch_env()?;
    hcl.fetch_meta()?;

    hcl.declare("engine", version);

    let lang = params.lang.unwrap_or(hcl.export.to_owned().unwrap_or_default());
    let file = hcl.file.to_owned().unwrap_or(file.rsplit_once('.').map(|(name, _)| name).unwrap_or(file).to_owned());

    let (data, ext) = match Language::parse(&lang) {
        Language::TOML => (hcl.toml(), "toml"),
        Language::JSON => (hcl.json(), "json"),
        Language::YAML => (hcl.yaml(), "yml"),
        Language::None => return Err(Error::from_str(400, "Language not found")),
    };

    res.set_body(data?);
    res.insert_header("Content-Disposition", format!(r#"attachment; filename="{file}.{ext}""#));

    Ok(res)
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
            res.set_body(format!("(message)\n{error}\n\n(error)\n{status}\n"));
        }
        Ok(res)
    }));

    app.at("/*path").get(compile);
    app.listen(config.settings.listen).await?;

    Ok(())
}
