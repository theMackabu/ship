use crate::declare_fns;

use chrono::{Duration, TimeZone, Utc};
use hcl::eval::{Context, FuncArgs};
use std::cell::RefMut;

pub fn init<'c>(mut ctx: RefMut<Context<'c>>) {
    declare_fns!(ctx, {
        timestamp => date::timestamp(),
        timeadd => date::timeadd(Number, String),
        parseduration => date::duration(String),
        formatdate => date::format(String, Number)
    });
}

fn timestamp(_args: FuncArgs) -> Result<hcl::Value, String> {
    let now = Utc::now().timestamp();
    Ok(hcl::Value::Number(hcl::Number::from_f64(now as f64).unwrap()))
}

fn timeadd(args: FuncArgs) -> Result<hcl::Value, String> {
    let timestamp = args[0].as_number().unwrap().as_f64().unwrap() as i64;
    let duration_str = args[1].as_str().unwrap();

    let duration = match parse_duration(duration_str) {
        Ok(d) => d,
        Err(e) => return Err(format!("Invalid duration: {}", e)),
    };

    let datetime = Utc.timestamp_opt(timestamp, 0).unwrap().checked_add_signed(duration).ok_or("Time overflow")?;

    Ok(hcl::Value::Number(hcl::Number::from_f64(datetime.timestamp() as f64).unwrap()))
}

fn parseduration(args: FuncArgs) -> Result<hcl::Value, String> {
    let duration_str = args[0].as_str().unwrap();

    let duration = match parse_duration(duration_str) {
        Ok(d) => d,
        Err(e) => return Err(format!("Invalid duration: {}", e)),
    };

    Ok(hcl::Value::Number(hcl::Number::from_f64(duration.num_seconds() as f64).unwrap()))
}

fn formatdate(args: FuncArgs) -> Result<hcl::Value, String> {
    let format = args[0].as_str().unwrap();
    let timestamp = args[1].as_number().unwrap().as_f64().unwrap() as i64;

    let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    Ok(hcl::Value::String(datetime.format(format).to_string()))
}

fn parse_duration(duration_str: &str) -> Result<Duration, String> {
    let mut chars = duration_str.chars().peekable();
    let mut value = String::new();
    let mut total = Duration::zero();

    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            value.push(ch);
            chars.next();
        } else {
            let num = value.parse::<i64>().map_err(|_| "Invalid duration number".to_string())?;
            value.clear();

            match chars.next() {
                Some('s') => total = total + Duration::seconds(num),
                Some('m') => total = total + Duration::minutes(num),
                Some('h') => total = total + Duration::hours(num),
                Some('d') => total = total + Duration::days(num),
                Some(unit) => return Err(format!("Invalid duration unit: {}", unit)),
                None => return Err("Duration string ended unexpectedly".to_string()),
            }
        }
    }

    Ok(total)
}
