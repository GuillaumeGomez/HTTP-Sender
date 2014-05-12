#![feature(globs)]

extern crate http_sender;

use http_sender::{HttpSender};
use std::os;

fn main() {
    let mut server;
    let mut page = "/".to_owned();

    let tmp = os::args();
    let args = tmp.tail();
    let mut values = Vec::new();

    if args.len() < 1 {
        fail!("USAGE: ./program server_name [page -> optional] [args_name=args_value...]\n");
    }

    match args {
        [ref a2] => {
            server = a2.to_owned();
        },
        [ref a2, ref a3] => {
            server = a2.to_owned();
            page = a3.to_owned();
            if page == "".to_owned() {
                fail!("page cannot be empty");
            }
        },
        _ => {
            server = args.get(0).unwrap().to_owned();
            page = args.get(1).unwrap().to_owned();
            if page == "".to_owned() {
                fail!("page cannot be empty");
            }
            for tmp in args.tailn(2).iter() {
                let segs = tmp.splitn('=', 1).collect::<Vec<&str>>();
                if segs.len() == 2 {
                    values.push((segs.get(0).trim().to_owned(), segs.get(1).trim().to_owned()));
                }
            }
        },
    }
    
    let mut h = if values.len() == 0 {
        HttpSender::new(server, page)
    } else {
        HttpSender::create_request(server, page, "POST").add_arguments(values)
    };
    match h.send_request() {
        Err(e) => println!("Error: {}", e),
        Ok(_) => match h.get_response() {
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