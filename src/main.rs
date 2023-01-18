#[macro_use]
extern crate fstrings;

extern crate base64;

extern crate clap;

#[macro_use]
extern crate serde_json;

use fstrings::f;
use rand::Rng;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;



use const_format::formatcp;
use clap::{Command, crate_version, Arg};

const PREFIX: &str = "\x1bPtmux;\x1b\x1bP?";
const SUFFIX: &str = "\x1b\\\x1b\\";

const PROTOCOL_MAJOR_VERSION: i32 = 0;
const PROTOCOL_MINOR_VERSION: i32 = 1;
const PROTOCOL_MINOR_SUBVERSION: i32 = 1;

const PROTOCOL_PREFIX: &str = formatcp!("{}{};{};{}",
                PREFIX, 
                PROTOCOL_MAJOR_VERSION,
                PROTOCOL_MINOR_VERSION,
                PROTOCOL_MINOR_SUBVERSION);

fn build_command_payload(random_id: i32, cmd: &str) -> String {
    let b64 = base64::encode(cmd);
    return f!("{PROTOCOL_PREFIX};{random_id};0z{b64}{SUFFIX}");
}

fn build_payload(random_id: i32, payload_number: u32, data: &[u8]) -> String {
    let b64 = base64::encode(data);
    return f!("{PROTOCOL_PREFIX};{random_id};{payload_number}z{b64}{SUFFIX}");
}

fn build_close_payload(random_id: i32) -> String {
    return f!("{PROTOCOL_PREFIX};{random_id};1z{SUFFIX}");
}

fn wrap_stdin(command: String) {
    let mut stdin = io::stdin();
    let mut ttyout = OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .expect("unable to open /dev/tty for writing");

    let mut buf: [u8; 65536] = [0; 65536];

    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;


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



fn main() {

    const ABOUT_TEXT: &str = "This is a helper to tell Zint to create a React Component and pass it the data from its stdin. \n\
        It will ask Zint terminal it create the requested <COMPONENT> (default: iframe)\n\
        and transmit data arriving on its STDIN to the component.\n\n\
        example : cat image.png | zint => will create an iframe that will display the image.png image file.";

    let app_m = Command::new("zint")
    .version(crate_version!())
    .about(ABOUT_TEXT)
    .allow_external_subcommands(true)
    .subcommand_value_name("COMPONENT [COMPONENT_OPTIONS]")
        .arg(Arg::new("title")
           .short('t')
           .long("title")
           .takes_value(true)
           .value_name("TITLE")
           .help("title of the tab"))
       /* .arg(Arg::new("position")
            .short('p')
            .help("position of the component")
            .possible_values(["top","bottom","tab"])
        )*/
    .get_matches();

    //println!("{:?}", app_m);

    let mut json_command = json!({});
    let subcommand = app_m.subcommand();

    if let Some(c) = app_m.value_of("title") {
        json_command["title"] = json!(c);
    }

    //println!("json object {}", json_command.to_string());

    //println!("{:?}", subcommand);
    json_command["command"] = match subcommand {
        None => { json!([String::from("")]) }
        Some((name, arg_matches)) => { 
            let mut params: Vec<String> = Vec::new();
            params.push(String::from(name));
            if let Some(subargs) = arg_matches.values_of("") {
                params.extend(subargs.map(|x| String::from(x)).collect::<Vec<String>>());
            }
            json!(params)
        }
    };


   wrap_stdin(json_command.to_string());
}
