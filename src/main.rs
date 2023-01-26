#[macro_use]
extern crate fstrings;

extern crate base64;

extern crate clap;

#[macro_use]
extern crate serde_json;

use futures::pin_mut;
use std::pin::Pin;


use fstrings::f;
use rand::Rng;
use std::env;
use std::os::fd::AsRawFd;

use std::time::Duration;


use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use async_std::io::{self, ReadExt, WriteExt, Stdout};

use tokio::time::sleep;



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


async fn wrap_stdin(command: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let ttyout = OpenOptions::new().write(true)/* .create(true) */.open("/dev/tty").await?;

    pin_mut!(ttyout);



    let mut buf: [u8; 65536] = [0; 65536];

    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;


    ttyout
        .write_all(build_command_payload(random_id, &command).as_bytes())
            .await.expect("unable to write command payload to /dev/tty");

    let mut payload_number: u32 = 10; // 0 to 9 are reserved for command and closing
    
    loop {
        let n = stdin.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        ttyout
            .write_all(build_payload(random_id, payload_number, &buf[..n]).as_bytes())
                .await.expect("unable to write payload to /dev/tty");
        payload_number += 1;
    }
             
    

    ttyout
        .write_all(build_close_payload(random_id).as_bytes())
            .await.expect("unable to write closing payload to /dev/tty");
    ttyout.flush().await?;

    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {

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


   wrap_stdin(json_command.to_string()).await?;


   Ok(())
}
