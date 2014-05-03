#![feature(globs)]

extern crate http_sender;

use http_sender::{HttpSender, ResponseData};
use std::os;

fn main() {
    let mut server;
    let mut page = ~"/";

    match os::args().as_slice() {
        [_, ref a2] => {
            server = a2.to_owned();
        },
        [_, ref a2, ref a3] => {
            if page == ~"" {
                fail!("page cannot be empty");
            }
            server = a2.to_owned();
            page = a3.to_owned();
        },
        _ => {
            fail!("USAGE: ./program server_name [page -> optional]\n")
        },
    }
    
    let mut h = HttpSender::new(server, page);
    match h.sendRequest("") {
        Err(e) => println!("Error: {}", e),
        Ok(_) => match h.getResponse() {
            Err(e) => println!("Error: {}", e),
            Ok(response) => {
                println!("Response from server:");
                println!("HTTP/{} {} {}\n", response.version, response.reason, response.status);
                for (v, k) in response.headers.iter() {
                    println!("{}: {}", v, k);
                }
                println!("\n{}", response.body)
            },
        },
    }
}