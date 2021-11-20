#[macro_use]
extern crate fstrings;

extern crate base64;

use std::env;
use std::io;
use rand::{Rng};
use std::io::prelude::*;
use std::fs::{self, OpenOptions};
use fstrings::f;


const PREFIX: &str = "\x1bPtmux;\x1b\x1bP?";
const SUFFIX: &str = "\x1b\\\x1b\\";

fn build_command_payload(random_id: i32, cmd: &str) -> String {
    return f!("{PREFIX}{random_id};0a{cmd}{SUFFIX}");
}

fn build_payload(random_id: i32, payload_number: u32, data: &[u8]) -> String {
    let b64 = base64::encode(data);
    return f!("{PREFIX}{random_id};{payload_number}a{b64}{SUFFIX}");
}

fn build_close_payload(random_id: i32) -> String {
    return f!("{PREFIX}{random_id};1a{SUFFIX}");
}


fn main() {
    let mut stdin = io::stdin();
    let mut ttyout = OpenOptions::new().write(true).open("/dev/tty").expect("unable to open /dev/tty for writing");
    
    let mut buf: [u8; 65536] = [0; 65536];

    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;

    let command = env::args().skip(1).collect::<Vec<String>>().join(" ");

    ttyout.write(build_command_payload(random_id, &command).as_bytes());

    loop {

        match stdin.read(&mut buf) {
            Ok(0) => { break; }
            Ok(i) => { 
                ttyout.write(build_payload(random_id, 2, &buf[..i]).as_bytes());
            }
            Err(e) => { panic!("{}", e) }
        }
    }

    ttyout.write(build_close_payload(random_id).as_bytes());
}