use crate::models::Config;
use macros_rs::fmt::{crashln, string};
use owo_colors::OwoColorize;
use std::fs;

pub(crate) fn read() -> Config {
    let contents = match fs::read_to_string("config.hcl") {
        Ok(contents) => contents,
        Err(err) => crashln!("Cannot find config.\n{}", string!(err).white()),
    };

    match hcl::from_str(&contents).map_err(|err| string!(err)) {
        Ok(parsed) => parsed,
        Err(err) => crashln!("Cannot parse config.\n{}", err.white()),
    }
}
