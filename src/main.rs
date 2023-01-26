#[macro_use]
extern crate fstrings;

extern crate base64;

extern crate clap;

#[macro_use]
extern crate serde_json;

extern crate atty;

mod echo;


use fstrings::f;
use rand::Rng;
use std::env;

use tokio::fs::{OpenOptions};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use async_std::io::{self, ReadExt, WriteExt};

use std::os::unix::io::AsRawFd;

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

fn build_eof_payload(random_id: i32) -> String {
    return f!("{PROTOCOL_PREFIX};{random_id};1z{SUFFIX}");
}

fn build_close_payload(random_id: i32) -> String {
    return f!("{PROTOCOL_PREFIX};{random_id};2z{SUFFIX}");
}



async fn wrap_stdin(command: String, 
        do_read_stdin: bool, 
        random_id: i32) -> io::Result<()> {
    let mut stdin = io::stdin();
    //let mut stdout = io::stdout();
    let mut ttyout = OpenOptions::new().write(true).open("/dev/tty").await?;


    let mut buf: [u8; 65536] = [0; 65536];

    ttyout
        .write_all(build_command_payload(random_id, &command).as_bytes())
            .await.expect("unable to write command payload to /dev/tty");

    let mut payload_number: u32 = 10; // 0 to 9 are reserved for command and closing
    
    if do_read_stdin {
        loop {
            let n = stdin.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            ttyout
                .write_all(
                    build_payload(random_id, payload_number, &buf[..n])
                    .as_bytes())
                .await
                .expect("unable to write payload to /dev/tty");
            payload_number += 1;
        }
    }
    else {
        ttyout.write_all(b"not reading stdin as it is a tty").
            await.expect("unable to write payload to /dev/tty");
    }

    ttyout
        .write_all(build_eof_payload(random_id).as_bytes())
            .await.expect("unable to write closing payload to /dev/tty");
    ttyout.flush().await?;

    Ok(())
}

async fn read_tty() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut ttyin = OpenOptions::new().
        read(true).
        write(false).open("/dev/tty").await?;

    let mut buf: [u8; 65536] = [0; 65536];

    let _disable_echo = echo::HiddenInput::new(ttyin.as_raw_fd());

    loop {
        let n = ttyin.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        let decoded = base64::decode(&buf[..n-1]).unwrap();
        stdout
            .write_all(&decoded)
            .await
            .expect("unable to write payload to stdout");
        }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()>  {

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

    let json_command = json_command.to_string();


    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;

    let wrap_handle = tokio::spawn(async move {
        wrap_stdin(json_command, 
            atty::isnt(atty::Stream::Stdin), 
            random_id).await?;

        Ok::<_, io::Error>(())
    });


    let read_handle = tokio::spawn(async move {
        read_tty().await?;

        Ok::<_, io::Error>(())
    });

    wrap_handle.await??;
    read_handle.await??;

    // would be nicer to not open the terminal twice but in the meantime...
    let mut ttyout = OpenOptions::new().write(true).open("/dev/tty").await?;
    ttyout.write_all(build_close_payload(random_id).as_bytes()).await?;
    
    Ok(())
}
