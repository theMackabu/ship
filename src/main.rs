#![recursion_limit = "256"]

mod config;
mod func;
mod models;

use macros_rs::{fmt::str, obj::lazy_lock};
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
        $($ctx.borrow_mut().declare_func($name, $fn.to_owned());)+
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

lazy_lock! {
    static FN_FORMAT: FuncDef = FuncDef::builder().variadic_param(ParamType::Any).build(func::format);
    static FN_UPPER: FuncDef = FuncDef::builder().param(ParamType::String).build(func::upper);
    static FN_LOWER: FuncDef = FuncDef::builder().param(ParamType::String).build(func::lower);
    static FN_CONCAT: FuncDef = FuncDef::builder().variadic_param(ParamType::String).build(func::concat);
    static FN_VEC: FuncDef = FuncDef::builder().variadic_param(ParamType::Any).build(func::vec);
    static FN_LENGTH: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::length);
    static FN_RANGE: FuncDef = FuncDef::builder().param(ParamType::Number).param(ParamType::Number).build(func::range);
    static FN_COMPACT: FuncDef = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::compact);
    static FN_TYPE_OF: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::type_of);
    static FN_REVERSE: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::reverse);
    static FN_SUM: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::sum);
    static FN_UNIQUE: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::unique);
    static FN_CONTAINS: FuncDef = FuncDef::builder().param(ParamType::Any).param(ParamType::Any).build(func::contains);
    static FN_KEYS: FuncDef = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::keys);
    static FN_VALUES: FuncDef = FuncDef::builder().param(ParamType::Object(Box::new(ParamType::Any))).build(func::values);
    static FN_SPLIT: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::split);
    static FN_JOIN: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).param(ParamType::String).build(func::join);
    static FN_MAX: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::max);
    static FN_MIN: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::min);
    static FN_FLATTEN: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::flatten);
    static FN_MERGE: FuncDef = FuncDef::builder().variadic_param(ParamType::Object(Box::new(ParamType::Any))).build(func::merge);
    static FN_FILE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::file);
    static FN_HTTP_GET: FuncDef = FuncDef::builder().param(ParamType::String).variadic_param(ParamType::Any).build(func::http_get);
    static FN_VAULT_GET: FuncDef = FuncDef::builder().param(ParamType::String).build(func::vault_kv);
    static FN_HTTP_POST: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).variadic_param(ParamType::Any).build(func::http_post);
    static FN_HTTP_POST_JSON: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::Any).variadic_param(ParamType::Any).build(func::http_post_json);
    static FN_HTTP_PUT: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).variadic_param(ParamType::Any).build(func::http_put);
    static FN_TRIMSPACE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::trimspace);
    static FN_TRIM: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trim);
    static FN_TRIMPREFIX: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trimprefix);
    static FN_TRIMSUFFIX: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::trimsuffix);
    static FN_ABS: FuncDef = FuncDef::builder().param(ParamType::Number).build(func::abs);
    static FN_CEIL: FuncDef = FuncDef::builder().param(ParamType::Number).build(func::ceil);
    static FN_FLOOR: FuncDef = FuncDef::builder().param(ParamType::Number).build(func::floor);
    static FN_TIMESTAMP: FuncDef = FuncDef::builder().build(func::timestamp);
    static FN_TIMEADD: FuncDef = FuncDef::builder().param(ParamType::Number).param(ParamType::String).build(func::timeadd);
    static FN_FORMATDATE: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::formatdate);
    static FN_BCRYPT: FuncDef = FuncDef::builder().param(ParamType::String).build(func::bcrypt_hash);
    static FN_FILEMD5: FuncDef = FuncDef::builder().param(ParamType::String).build(func::filemd5);
    static FN_FILESHA1: FuncDef = FuncDef::builder().param(ParamType::String).build(func::filesha1);
    static FN_FILESHA256: FuncDef = FuncDef::builder().param(ParamType::String).build(func::filesha256);
    static FN_FILESHA512: FuncDef = FuncDef::builder().param(ParamType::String).build(func::filesha512);
    static FN_MD5: FuncDef = FuncDef::builder().param(ParamType::String).build(func::md5_hash);
    static FN_SHA1: FuncDef = FuncDef::builder().param(ParamType::String).build(func::sha1_hash);
    static FN_SHA256: FuncDef = FuncDef::builder().param(ParamType::String).build(func::sha256_hash);
    static FN_SHA512: FuncDef = FuncDef::builder().param(ParamType::String).build(func::sha512_hash);
    static FN_UUID: FuncDef = FuncDef::builder().build(func::uuid_gen);
    static FN_UUIDV5: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::String).build(func::uuidv5);
    static FN_BASE64ENCODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::base64encode);
    static FN_BASE64DECODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::base64decode);
    static FN_JSONENCODE: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::jsonencode);
    static FN_JSONDECODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::jsondecode);
    static FN_URLENCODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::urlencode);
    static FN_URLDECODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::urldecode);
    static FN_YAMLENCODE: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::yamlencode);
    static FN_YAMLDECODE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::yamldecode);
    static FN_TOSTRING: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::tostring);
    static FN_TONUMBER: FuncDef = FuncDef::builder().param(ParamType::Any).build(func::tonumber);
    static FN_TOSET: FuncDef = FuncDef::builder().param(ParamType::Array(Box::new(ParamType::Any))).build(func::toset);
    static FN_PARSEINT: FuncDef = FuncDef::builder().param(ParamType::String).build(func::parseint);
    static FN_CIDRNETMASK: FuncDef = FuncDef::builder().param(ParamType::String).build(func::cidrnetmask);
    static FN_CIDRRANGE: FuncDef = FuncDef::builder().param(ParamType::String).build(func::cidrrange);
    static FN_CIDRHOST: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::cidrhost);
    static FN_CIDRSUBNETS: FuncDef = FuncDef::builder().param(ParamType::String).param(ParamType::Number).build(func::cidrsubnets);
}

impl<'c> HclConverter<'c> {
    pub fn new(input: &str) -> Result<Self, Error> {
        let default = Self {
            file: None,
            export: None,
            data: input.to_owned(),
            ctx: Rc::new(RefCell::new(Context::new())),
        };

        declare_fns!(default.ctx,
            FN_VEC => "s",
            FN_JOIN => "join",
            FN_RANGE => "range",
            FN_MERGE => "merge",
            FN_SPLIT => "split",
            FN_FORMAT => "format",
            FN_CONCAT => "concat",
            FN_LENGTH => "length",
            FN_UNIQUE => "unique",
            FN_COMPACT => "compact",
            FN_TYPE_OF => "type_of",
            FN_REVERSE => "reverse",
            FN_FLATTEN => "flatten",
            FN_CONTAINS => "contains"
        );

        declare_fns!(default.ctx,
            FN_ABS => "abs",
            FN_CEIL => "ceil",
            FN_FLOOR => "floor",
            FN_MAX => "max",
            FN_MIN => "min",
            FN_SUM => "sum",
            FN_PARSEINT => "parseint"
        );

        declare_fns!(default.ctx,
            FN_FILE => name!("fs", "read"),
            FN_FILEMD5 => name!(["fs", "hash"] => "md5"),
            FN_FILESHA1 => name!(["fs", "hash"] => "sha1"),
            FN_FILESHA256 => name!(["fs", "hash"] => "sha256"),
            FN_FILESHA512 => name!(["fs", "hash"] => "sha512")
        );

        declare_fns!(default.ctx,
            FN_HTTP_GET => name!("http", "get"),
            FN_VAULT_GET => name!("secret", "kv"),
            FN_HTTP_POST => name!("http", "post"),
            FN_HTTP_POST_JSON => name!("http", "post_json"),
            FN_HTTP_PUT => name!("http", "put")
        );

        declare_fns!(default.ctx,
            FN_KEYS => name!("map", "keys"),
            FN_VALUES => name!("map", "values"),
            FN_UPPER => name!("str", "upper"),
            FN_LOWER => name!("str", "lower"),
            FN_TRIM => name!("str", "trim"),
            FN_TRIMSPACE => name!("str", "trimspace"),
            FN_TRIMPREFIX => name!("str", "trimprefix"),
            FN_TRIMSUFFIX => name!("str", "trimsuffix")
        );

        declare_fns!(default.ctx,
            FN_TIMESTAMP => name!("date", "timestamp"),
            FN_TIMEADD => name!("date", "timeadd"),
            FN_FORMATDATE => name!("date", "format")
        );

        declare_fns!(default.ctx,
            FN_BCRYPT => name!("hash", "bcrypt"),
            FN_MD5 => name!("hash", "md5"),
            FN_SHA1 => name!("hash", "sha1"),
            FN_SHA256 => name!("hash", "sha256"),
            FN_SHA512 => name!("hash", "sha512")
        );

        declare_fns!(default.ctx,
           FN_UUID => "uuid",
           FN_UUIDV5 => "uuidv5"
        );

        declare_fns!(default.ctx,
           FN_VEC => "list",
           FN_VEC => "tuple",
           FN_TOSET => "set",
           FN_TOSTRING => "string",
           FN_TONUMBER => "number"
        );

        declare_fns!(default.ctx,
           FN_BASE64ENCODE => name!("encode", "base64"),
           FN_BASE64DECODE => name!("decode", "base64"),
           FN_JSONENCODE => name!("encode", "json"),
           FN_JSONDECODE => name!("decode", "json"),
           FN_URLENCODE => name!("encode", "url"),
           FN_URLDECODE => name!("decode", "url"),
           FN_YAMLENCODE => name!("encode", "yaml"),
           FN_YAMLDECODE => name!("decode", "yaml")
        );

        declare_fns!(default.ctx,
           FN_CIDRSUBNETS => name!("cidr", "subnets"),
           FN_CIDRNETMASK => name!("cidr", "netmask"),
           FN_CIDRRANGE => name!("cidr", "range"),
           FN_CIDRHOST => name!("cidr", "host")
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

        if let Some(locals) = locals {
            self.declare("local", locals.to_owned());
        }

        let var = obj.get("var").and_then(|m| m.as_object());
        let vars = obj.get("vars").and_then(|m| m.as_object());
        let let_block = obj.get("let").and_then(|m| m.as_object());
        let const_block = obj.get("const").and_then(|m| m.as_object());

        let mut combined = hcl::Map::new();

        if let Some(const_map) = const_block {
            combined.extend(const_map.to_owned());
        }

        let check_const_conflicts = |map: &hcl::Map<String, hcl::Value>, block_name: &str| -> Result<(), Error> {
            if let Some(const_map) = const_block {
                let conflicting_keys: Vec<String> = map.keys().filter(|k| const_map.contains_key(*k)).map(|k| k.to_string()).collect();

                let err_msg = format!("Cannot override const values in '{}' block for keys: {}", block_name, conflicting_keys.join(", "));

                if !conflicting_keys.is_empty() {
                    return Err(Error::from_str(500, str!(err_msg)));
                }
            }
            Ok(())
        };

        if let Some(var_map) = var {
            check_const_conflicts(var_map, "var")?;
            combined.extend(var_map.to_owned());
        }

        if let Some(let_map) = let_block {
            check_const_conflicts(let_map, "let")?;
            combined.extend(let_map.to_owned());
        }

        if let Some(vars_map) = vars {
            check_const_conflicts(vars_map, "vars")?;

            let conflicting_keys: Vec<String> = vars_map.keys().filter(|k| combined.contains_key(*k)).map(|k| k.to_string()).collect();

            let err_msg = format!("Conflicting variables in 'vars' block for keys: {}", conflicting_keys.join(", "));

            if !conflicting_keys.is_empty() {
                return Err(Error::from_str(500, str!(err_msg)));
            }

            combined.extend(vars_map.to_owned());
        }

        if !combined.is_empty() {
            self.declare("var", combined);
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
            obj.shift_remove("locals");
            obj.shift_remove("meta");
            obj.shift_remove("const");
            obj.shift_remove("let");
            obj.shift_remove("var");
            obj.shift_remove("vars");
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

    hcl.declare("boolean", true);
    hcl.declare("number", 0);
    hcl.declare("string", "");
    hcl.declare("null", hcl::Value::Null);
    hcl.declare("object", hcl::Map::new());
    hcl.declare::<&str, Vec<String>>("array", vec![]);

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
