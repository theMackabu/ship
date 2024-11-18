use crate::declare_fns;

use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

use bcrypt::{hash, DEFAULT_COST};
use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use uuid::Uuid;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        uuid_gen => uuid(),
        uuidv5 => uuidv5(String, String),
        bcrypt_hash => hash::bcrypt(String),
        md5_hash => hash::md5(String),
        sha1_hash => hash::sha1(String),
        sha256_hash => hash::sha256(String),
        sha512_hash => hash::sha512(String)
    });
}

fn uuid_gen(_args: FuncArgs) -> Result<hcl::Value, String> { Ok(hcl::Value::String(Uuid::new_v4().to_string())) }

fn uuidv5(args: FuncArgs) -> Result<hcl::Value, String> {
    let namespace = args[0].as_str().unwrap();
    let name = args[1].as_str().unwrap();

    let namespace_uuid = Uuid::parse_str(namespace).map_err(|e| format!("Invalid namespace UUID: {}", e))?;
    let uuid = Uuid::new_v5(&namespace_uuid, name.as_bytes());

    Ok(hcl::Value::String(uuid.to_string()))
}

fn bcrypt_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    match hash(input.as_bytes(), DEFAULT_COST) {
        Ok(hashed) => Ok(hcl::Value::String(hashed)),
        Err(e) => Err(format!("Bcrypt error: {}", e)),
    }
}

fn md5_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

fn sha1_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

fn sha256_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}

fn sha512_hash(args: FuncArgs) -> Result<hcl::Value, String> {
    let input = args[0].as_str().unwrap();
    let mut hasher = Sha512::new();
    hasher.update(input.as_bytes());
    Ok(hcl::Value::String(format!("{:x}", hasher.finalize())))
}
