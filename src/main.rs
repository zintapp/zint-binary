#[macro_use]
extern crate fstrings;

extern crate base64;

use fstrings::f;
use rand::Rng;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;

const PREFIX: &str = "\x1bPtmux;\x1b\x1bP?";
const SUFFIX: &str = "\x1b\\\x1b\\";

const PROTOCOL_VERSION: i32 = 1;

fn build_command_payload(random_id: i32, cmd: &str) -> String {
    return f!("{PREFIX}{PROTOCOL_VERSION};{random_id};0z{cmd}{SUFFIX}");
}

fn build_payload(random_id: i32, payload_number: u32, data: &[u8]) -> String {
    let b64 = base64::encode(data);
    return f!("{PREFIX}{PROTOCOL_VERSION};{random_id};{payload_number}z{b64}{SUFFIX}");
}

fn build_close_payload(random_id: i32) -> String {
    return f!("{PREFIX}{PROTOCOL_VERSION};{random_id};1z{SUFFIX}");
}

fn main() {
    let mut stdin = io::stdin();
    let mut ttyout = OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .expect("unable to open /dev/tty for writing");

    let mut buf: [u8; 65536] = [0; 65536];

    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;

    let command = env::args().skip(1).collect::<Vec<String>>().join(" ");

    ttyout
        .write(build_command_payload(random_id, &command).as_bytes())
        .expect("unable to write zint command to /dev/tty");

    let mut payload_number: i32 = 2; // 0 and 1 are reserved for command and closing

    loop {
        match stdin.read(&mut buf).expect("reading from stdin") {
            0 => {
                break; /* stdin has received EOF and is closed */
            }
            i => {
                ttyout
                    .write(build_payload(random_id, payload_number as u32, &buf[..i]).as_bytes())
                    .expect("unable to write stdin payload to /dev/tty");
                payload_number += 1;
                if payload_number < 0 {
                    payload_number = 2;
                }
            }
        }
    }

    ttyout
        .write(build_close_payload(random_id).as_bytes())
        .expect("unable to write closing payload to /dev/tty");
}
