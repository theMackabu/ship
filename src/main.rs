use std::cell::RefCell;
use std::{error::Error, fs, rc::Rc, str::FromStr};

use hcl::eval::{Context, Evaluate};
use hcl::Template;

use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

pub struct HclConverter<'c> {
    data: String,
    ctx: Rc<RefCell<Context<'c>>>,
}

impl<'c> HclConverter<'c> {
    pub fn new(input: &str) -> Result<Self, Box<dyn Error>> {
        let default = Self {
            data: input.to_owned(),
            ctx: Rc::new(RefCell::new(Context::new())),
        };

        Ok(default)
    }

    pub fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        Self::new(&content)
    }

    pub fn declare<I, T>(&mut self, name: I, value: T)
    where
        I: Into<hcl::Identifier>,
        T: Into<hcl::Value>,
    {
        self.ctx.borrow_mut().declare_var(name.into(), value.into());
    }

    pub fn eval(&self) -> Result<hcl::Value, Box<dyn Error>> {
        let tmpl = Template::from_str(&self.data)?;
        let data = tmpl.evaluate(&self.ctx.borrow())?;

        Ok(hcl::from_str(&data)?)
    }

    pub fn toml(&self) -> Result<String, Box<dyn Error>> {
        let value = self.to_toml(&self.eval()?);
        Ok(toml::to_string_pretty(&value)?)
    }

    pub fn yaml(&self) -> Result<String, Box<dyn Error>> {
        let value = self.to_yaml(&self.eval()?);
        Ok(serde_yaml_ng::to_string(&value)?)
    }

    pub fn json(&self) -> Result<String, Box<dyn Error>> {
        let value = self.to_json(&self.eval()?);
        Ok(serde_json::to_string_pretty(&value)?)
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut converter = HclConverter::read("test.hcl")?;

    converter.declare("domain", "themackabu.dev");

    // Convert to TOML
    let toml_string = converter.toml()?;
    println!("TOML:\n{}\n", toml_string);

    // Convert to YAML
    let yaml_string = converter.yaml()?;
    println!("YAML:\n{}\n", yaml_string);

    // Convert to JSON
    let json_string = converter.json()?;
    println!("JSON:\n{}", json_string);

    Ok(())
}
