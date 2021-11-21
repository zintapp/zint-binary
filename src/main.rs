

#[macro_use]
extern crate fstrings;

extern crate base64;


use futures::pin_mut;
use std::env;
use std::pin::Pin;
use rand::{Rng};
use fstrings::f;


use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

use std::time::Duration;



use async_std::io::{stdin, stdout, ReadExt, WriteExt, Stdout};

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let mut stdin = stdin();
    let mut stdout = stdout();

    let ttyout = OpenOptions::new().write(true)/* .create(true) */.open("/dev/tty").await?;

    pin_mut!(ttyout);
        

    let mut buf: [u8; 65536] = [0; 65536];

    let random_id_u: u32 = rand::thread_rng().gen();
    let random_id: i32 = random_id_u as i32 & i32::MAX;

    let command = env::args().skip(1).collect::<Vec<String>>().join(" ");

    ttyout.write_all(build_command_payload(random_id, &command).as_bytes()).await?;

    loop {
        match stdin.read(&mut buf).await? {
            0 => {
                debug(&mut stdout, &mut ttyout, "received 0 bytes!\n".as_bytes()).await?;

                break;
            }
            i => { 
                debug(&mut stdout, &mut ttyout, f!("received {i} bytes!\n").as_bytes()).await?;
                match ttyout.write_all(build_payload(random_id, 2, &buf[..i]).as_bytes()).await {
                    Ok(()) => {}
                    Err(e) => { panic!("Error on write! {}", e)}
                }
            }
        }
    }
    
    //stdout.write(f!("printing closing payload!\n").as_bytes()).await?;

    ttyout.write_all(build_close_payload(random_id).as_bytes()).await?;
    ttyout.flush().await?;
    Ok(())
}

async fn debug(stdout: &mut Stdout, ttyout: &mut Pin<&mut File>, buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    ttyout.flush().await?;
    stdout.write(buf).await?;
    stdout.flush().await?;
    Ok(())
}