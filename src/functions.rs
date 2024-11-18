mod cidr;
mod convert;
mod crypto;
mod date;
mod file;
mod global;
mod hash;
mod http;
mod num;
mod string;

use hcl::eval::Context;
use std::{cell::RefCell, rc::Rc};

pub type Functions<'c> = Rc<RefCell<Context<'c>>>;

pub fn init<'c>() -> Functions<'c> {
    let ctx = Rc::new(RefCell::new(Context::new()));

    cidr::init(ctx.borrow_mut());
    convert::init(ctx.borrow_mut());
    crypto::init(ctx.borrow_mut());
    date::init(ctx.borrow_mut());
    file::init(ctx.borrow_mut());
    global::init(ctx.borrow_mut());
    hash::init(ctx.borrow_mut());
    http::init(ctx.borrow_mut());
    num::init(ctx.borrow_mut());
    string::init(ctx.borrow_mut());

    return ctx;
}
