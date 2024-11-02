mod config;
mod func;
mod models;

use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, str::FromStr};

use hcl::eval::{Context, FuncDef, ParamType};
use hcl::expr::FuncName;
use hcl::Block;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

use tide::{utils::After, Error, Request, Response};
use tide_tracing::TraceMiddleware;

macro_rules! declare_fns {
    ($ctx:expr, $($fn:expr => $name:expr),+ $(,)?) => {
        $($ctx.borrow_mut().declare_func($name, $fn);)+
    };
}

macro_rules! name {
    ($name:expr) => {
        FuncName::new($name)
    };
    ($namespace:expr, $name:expr) => {
        FuncName::new($name).with_namespace([$namespace])
    };
    ([$($ns:expr),+] => $name:expr) => {
        FuncName::new($name).with_namespace(vec![$($ns),+])
    };
}

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

        let fn_format = FuncDef::builder().variadic_param(ParamType::Any).build(func::format);
        let fn_upper = FuncDef::builder().param(ParamType::String).build(func::upper);
        let fn_lower = FuncDef::builder().param(ParamType::String).build(func::lower);
        let fn_concat = FuncDef::builder().variadic_param(ParamType::String).build(func::concat);
        let fn_vec = FuncDef::builder().variadic_param(ParamType::Any).build(func::vec);
        let fn_length = FuncDef::builder().param(ParamType::Any).build(func::length);
        let fn_range = FuncDef::builder().param(ParamType::Number).param(ParamType::Number).build(func::range);
        let fn_compact = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::compact);
        let fn_type_of = FuncDef::builder().param(ParamType::Any).build(func::type_of);
        let fn_reverse = FuncDef::builder().param(ParamType::Any).build(func::reverse);
        let fn_sum = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::sum);
        let fn_unique = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::unique);
        let fn_contains = FuncDef::builder().param(ParamType::Any).param(ParamType::Any).build(func::contains);
        let fn_keys = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::keys);
        let fn_values = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::values);
        let fn_split = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::split);
        let fn_join = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).param(ParamType::String).build(func::join);
        let fn_max = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::max);
        let fn_min = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::min);
        let fn_flatten = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::flatten);
        let fn_merge = FuncDef::builder().variadic_param(ParamType::Object(Box::new(ParamType::Any))).build(func::merge);
        let fn_file = FuncDef::builder().param(ParamType::String).build(func::file);
        let fn_http_get = FuncDef::builder().param(ParamType::String).build(func::http_get);
        let fn_http_post = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::http_post);
        let fn_http_post_json = FuncDef::builder().param(ParamType::String).param(ParamType::Any).build(func::http_post_json);
        let fn_http_put = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::http_put);
        let fn_trimspace = FuncDef::builder().param(ParamType::String).build(func::trimspace);
        let fn_trim = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trim);
        let fn_trimprefix = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trimprefix);
        let fn_trimsuffix = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trimsuffix);
        let fn_abs = FuncDef::builder().param(ParamType::Number).build(func::abs);
        let fn_ceil = FuncDef::builder().param(ParamType::Number).build(func::ceil);
        let fn_floor = FuncDef::builder().param(ParamType::Number).build(func::floor);
        let fn_timestamp = FuncDef::builder().build(func::timestamp);
        let fn_timeadd = FuncDef::builder().param(ParamType::Number).param(ParamType::String).build(func::timeadd);
        let fn_formatdate = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::formatdate);
        let fn_bcrypt = FuncDef::builder().param(ParamType::String).build(func::bcrypt_hash);
        let fn_filemd5 = FuncDef::builder().param(ParamType::String).build(func::filemd5);
        let fn_filesha1 = FuncDef::builder().param(ParamType::String).build(func::filesha1);
        let fn_filesha256 = FuncDef::builder().param(ParamType::String).build(func::filesha256);
        let fn_filesha512 = FuncDef::builder().param(ParamType::String).build(func::filesha512);
        let fn_md5 = FuncDef::builder().param(ParamType::String).build(func::md5_hash);
        let fn_sha1 = FuncDef::builder().param(ParamType::String).build(func::sha1_hash);
        let fn_sha256 = FuncDef::builder().param(ParamType::String).build(func::sha256_hash);
        let fn_sha512 = FuncDef::builder().param(ParamType::String).build(func::sha512_hash);
        let fn_uuid = FuncDef::builder().build(func::uuid_gen);
        let fn_uuidv5 = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::uuidv5);
        let fn_base64encode = FuncDef::builder().param(ParamType::String).build(func::base64encode);
        let fn_base64decode = FuncDef::builder().param(ParamType::String).build(func::base64decode);
        let fn_jsonencode = FuncDef::builder().param(ParamType::Any).build(func::jsonencode);
        let fn_jsondecode = FuncDef::builder().param(ParamType::String).build(func::jsondecode);
        let fn_urlencode = FuncDef::builder().param(ParamType::String).build(func::urlencode);
        let fn_urldecode = FuncDef::builder().param(ParamType::String).build(func::urldecode);
        let fn_yamlencode = FuncDef::builder().param(ParamType::Any).build(func::yamlencode);
        let fn_yamldecode = FuncDef::builder().param(ParamType::String).build(func::yamldecode);
        let fn_tostring = FuncDef::builder().param(ParamType::Any).build(func::tostring);
        let fn_tonumber = FuncDef::builder().param(ParamType::Any).build(func::tonumber);
        let fn_toset = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::toset);
        let fn_parseint = FuncDef::builder().param(ParamType::String).build(func::parseint);
        let fn_cidrnetmask = FuncDef::builder().param(ParamType::String).build(func::cidrnetmask);
        let fn_cidrrange = FuncDef::builder().param(ParamType::String).build(func::cidrrange);
        let fn_cidrhost = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::cidrhost);
        let fn_cidrsubnets = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::cidrsubnets);

        declare_fns!(default.ctx,
            fn_vec => "s",
            fn_join => "join",
            fn_range => "range",
            fn_merge => "merge",
            fn_split => "split",
            fn_format => "format",
            fn_concat => "concat",
            fn_length => "length",
            fn_unique => "unique",
            fn_compact => "compact",
            fn_type_of => "type_of",
            fn_reverse => "reverse",
            fn_flatten => "flatten",
            fn_contains => "contains"
        );

        declare_fns!(default.ctx,
            fn_abs => "abs",
            fn_ceil => "ceil",
            fn_floor => "floor",
            fn_max => "max",
            fn_min => "min",
            fn_sum => "sum",
            fn_parseint => "parseint"
        );

        declare_fns!(default.ctx,
            fn_file => name!("fs", "read"),
            fn_filemd5 => name!(["fs", "hash"] => "md5"),
            fn_filesha1 => name!(["fs", "hash"] => "sha1"),
            fn_filesha256 => name!(["fs", "hash"] => "sha256"),
            fn_filesha512 => name!(["fs", "hash"] => "sha512")
        );

        declare_fns!(default.ctx,
            fn_http_get => name!("http", "get"),
            fn_http_post => name!("http", "post"),
            fn_http_post_json => name!("http", "post_json"),
            fn_http_put => name!("http", "put")
        );

        declare_fns!(default.ctx,
            fn_keys => name!("map", "keys"),
            fn_values => name!("map", "values"),
            fn_upper => name!("str", "upper"),
            fn_lower => name!("str", "lower"),
            fn_trim => name!("str", "trim"),
            fn_trimspace => name!("str", "trimspace"),
            fn_trimprefix => name!("str", "trimprefix"),
            fn_trimsuffix => name!("str", "trimsuffix")
        );

        declare_fns!(default.ctx,
            fn_timestamp => name!("date", "timestamp"),
            fn_timeadd => name!("date", "timeadd"),
            fn_formatdate => name!("date", "format")
        );

        declare_fns!(default.ctx,
            fn_bcrypt => name!("hash", "bcrypt"),
            fn_md5 => name!("hash", "md5"),
            fn_sha1 => name!("hash", "sha1"),
            fn_sha256 => name!("hash", "sha256"),
            fn_sha512 => name!("hash", "sha512")
        );

        declare_fns!(default.ctx,
           fn_uuid => "uuid",
           fn_uuidv5 => "uuidv5"
        );

        declare_fns!(default.ctx,
           fn_base64encode => name!("encode", "base64"),
           fn_base64decode => name!("decode", "base64"),
           fn_jsonencode => name!("encode", "json"),
           fn_jsondecode => name!("decode", "json"),
           fn_urlencode => name!("encode", "url"),
           fn_urldecode => name!("decode", "url"),
           fn_yamlencode => name!("encode", "yaml"),
           fn_yamldecode => name!("decode", "yaml")
        );

        declare_fns!(default.ctx,
           fn_tostring => name!("to", "string"),
           fn_tonumber => name!("to", "number"),
           fn_toset => name!("to", "set")
        );

        declare_fns!(default.ctx,
           fn_cidrsubnets => name!("cidr", "subnets"),
           fn_cidrnetmask => name!("cidr", "netmask"),
           fn_cidrrange => name!("cidr", "range"),
           fn_cidrhost => name!("cidr", "host")
        );

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

    pub fn fetch_locals(&mut self) -> Result<(), Error> {
        let value: hcl::Value = hcl::from_str(&self.data)?;
        let obj = value.as_object().ok_or(Error::from_str(500, "Invalid root object"))?;

        let locals = obj.get("locals").and_then(|m| m.as_object());
        let env = obj.get("env").and_then(|m| m.as_object());

        if let Some(locals) = locals {
            self.declare("local", locals.to_owned());
        }

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

        if let Some(path) = file {
            let (name, extension) = match path.rsplit_once('.') {
                Some((name, ext)) => (name.to_string(), Some(ext.to_string())),
                None => (path, meta.get("export").and_then(|m| m.as_str()).map(|s| s.to_string())),
            };

            self.file = Some(name);
            self.export = extension;
        }

        Ok(self.declare("meta", meta.to_owned()))
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

    fn eval(&self) -> Result<hcl::Value, Error> { Ok(hcl::eval::from_str(&self.data, &self.ctx.borrow())?) }

    fn result(&self) -> Result<hcl::Value, Error> {
        let mut value = self.eval()?;

        if let hcl::Value::Object(obj) = &mut value {
            obj.shift_remove("meta");
            obj.shift_remove("locals");
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

    hcl.fetch_locals()?;
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
