#[macro_use]
extern crate fstrings;

extern crate base64;

extern crate clap;

use fstrings::f;
use rand::Rng;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;

use clap::{Command, crate_version};

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
/*
    let mut arg_vec = env::args().collect::<Vec<String>>();

    let split_ix = arg_vec.iter().skip(1).by_ref().position(| x | !x.starts_with("-")).unwrap_or(arg_vec.len()-1) + 1;

    let (zint_parameters, component_parameters) = arg_vec.split_at_mut(split_ix);
 
    println!("Zint arguments {:?}", zint_parameters);
    println!("Component arguments {:?}", component_parameters);
*/

    const ABOUT_TEXT: &str = "This is a helper to tell Zint to create a React Component and pass it the data from its stdin. \n\
        It will ask Zint terminal it create the requested <COMPONENT> (default: iframe)\n\
        and transmit data arriving on its STDIN to the component.\n\n\
        example : cat image.png | zint => will create an iframe that will display the image.png image file.";

    let app_m = Command::new("zint")
    .version(crate_version!())
    .about(ABOUT_TEXT)
    .allow_external_subcommands(true)
    .subcommand_value_name("COMPONENT [COMPONENT_OPTIONS]")
       /* .arg(Arg::new("position")
            .short('p')
            .help("position of the component")
            .possible_values(["top","bottom","tab"])
        )*/
    .get_matches();

    let subc = app_m.subcommand();


    let component_command = match app_m.subcommand() {
        None => { String::from("") }
        Some((name, arg_matches)) => { 
            let mut params: Vec<String> = Vec::new();
            params.push(String::from(name));
            if let Some(mut subargs) = arg_matches.values_of("") {
                params.extend(subargs.map(|x| String::from(x)).collect::<Vec<String>>());
            }
            params.join(" ")
        }
    };

   wrap_stdin(component_command);
}
