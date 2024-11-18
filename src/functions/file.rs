use crate::declare_fns;
use hcl::eval::{Context, FuncArgs};

use std::cell::RefMut;
use std::fs::{self, File};
use std::io::Read;

use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        file => fs::read(String),
        filemd5 => fs::md5(String),
        filesha1 => fs::sha1(String),
        filesha256 => fs::sha256(String),
        filesha512 => fs::sha512(String)
    });
}

fn file(args: FuncArgs) -> Result<hcl::Value, String> {
    let path = args[0].as_str().unwrap();
    match fs::read_to_string(path) {
        Ok(contents) => Ok(hcl::Value::String(contents)),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

fn filemd5(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn filesha1(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn filesha256(args: FuncArgs) -> Result<hcl::Value, String> {
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

fn filesha512(args: FuncArgs) -> Result<hcl::Value, String> {
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
