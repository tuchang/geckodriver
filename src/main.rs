//Until it's clear what the unstable things are replaced by
#![allow(unstable)]
#![allow(non_snake_case)]

#[macro_use] extern crate log;
extern crate "rustc-serialize" as rustc_serialize;
extern crate core;
extern crate getopts;
extern crate hyper;
extern crate regex;
#[macro_use] extern crate webdriver;

use getopts::{usage,optflag, getopts, OptGroup};
use std::io::net::ip::SocketAddr;
use std::str::FromStr;
use std::io;
use std::os;

use webdriver::error::WebDriverError;
use webdriver::server::start;

use marionette::MarionetteHandler;

macro_rules! try_opt {
    ($expr:expr, $err_type:expr, $err_msg:expr) => ({
        match $expr {
            Some(x) => x,
            None => return Err(WebDriverError::new($err_type, $err_msg))
        }
    })
}

mod marionette;

static DEFAULT_ADDR: &'static str = "127.0.0.1:4444";
static VERSION: &'static str = include_str!("../.version");

fn err(msg: String) {
    io::stderr().write_line(format!("{}: error: {}", os::args()[0], msg).as_slice()).unwrap();
}

fn print_usage(opts: &[OptGroup]) {
    let shorts: Vec<String> = opts.iter().map(|opt| opt.short_name.to_string()).collect();
    let msg = format!("usage: {} [-{}] [ADDRESS]", os::args()[0], shorts.concat());
    io::stderr().write_line(usage(msg.as_slice(), opts).as_slice()).unwrap();
}

// Valid addresses to parse are "HOST:PORT" or ":PORT".
// If the host isn't specified, 127.0.0.1 will be assumed.
fn parse_addr(s: String) -> Result<SocketAddr, String> {
    let mut parts: Vec<&str> = s.as_slice().splitn(1, ':').collect();
    if parts.len() == 2 {
        parts[0] = "127.0.0.1";
    }
    let full_addr = parts.connect(":");
    match FromStr::from_str(full_addr.as_slice()) {
        Some(addr) => Ok(addr),
        None => Err(format!("illegal address: {}", s))
    }
}

fn run(args: Vec<String>) -> Result<(), isize> {
    let opts = [
        optflag("q", "", "make the program quiet, only printing warnings"),
        optflag("v", "", "show version information"),
        optflag("h", "", "show this message"),
    ];
    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(f) => {
            err(format!("{}", f));
            return Ok(())
        }
    };

    if matches.opt_present("v") {
        println!("wires version {}", VERSION);
        return Ok(())
    } else if matches.opt_present("h") {
        print_usage(&opts);
        return Err(127);
    }

    let addr_str = if matches.free.len() == 1 {
        matches.free[0].clone()
    } else if matches.free.len() > 1 {
        err(format!("got {} positional arguments, expected 1", matches.free.len()));
        print_usage(&opts);
        return Err(1);
    } else {
        DEFAULT_ADDR.to_string()
    };
    let addr = match parse_addr(addr_str) {
        Ok(x) => x,
        Err(e) => {
            err(format!("{}", e));
            return Err(1);
        }
    };

    start(addr.ip, addr.port, MarionetteHandler::new());
    return Ok(());
}

fn main() {
    let args = os::args();
    run(args).unwrap_or_else(|x| os::set_exit_status(x));
}
