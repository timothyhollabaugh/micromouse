use std::error::Error;
use std::io::stdin;
use std::io::BufRead;
use std::io::Read;

use postcard;

use mouse::comms::DebugPacket;

fn main() {
    let mut buf = Vec::new();
    for b in stdin().bytes() {
        match b {
            Ok(byte) => {
                //println!("0x{:02x}", byte);
                buf.push(byte);
                match postcard::take_from_bytes::<DebugPacket>(&buf) {
                    Ok((debug, remaining)) => {
                        println!("{:#?}", debug);
                        buf = Vec::from(remaining.clone());
                    }
                    Err(postcard::Error::DeserializeUnexpectedEnd) => {}
                    Err(e) => println!("{:?}", e),
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
