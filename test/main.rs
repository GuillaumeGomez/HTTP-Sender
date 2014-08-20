#![feature(globs)]

extern crate http_sender;

use http_sender::{HttpSender};
use std::os;

fn main() {
    let mut server;
    let mut page = "/".to_string();

    let tmp = os::args();
    let args = tmp.tail();
    let mut values = Vec::new();

    if args.len() < 1 {
        fail!("USAGE: ./program server_name [page -> optional] [args_name=args_value...]\n");
    }

    match args {
        [ref a2] => {
            server = a2.to_string();
        },
        [ref a2, ref a3] => {
            server = a2.to_string();
            page = a3.to_string();
            if page == "".to_string() {
                fail!("page cannot be empty");
            }
        },
        _ => {
            server = args.get(0).unwrap().to_string();
            page = args.get(1).unwrap().to_string();
            if page == "".to_string() {
                fail!("page cannot be empty");
            }
            for tmp in args.slice_from(2).iter() {
                let segs = tmp.as_slice().splitn(1, '=').collect::<Vec<&str>>();
                if segs.len() == 2 {
                    values.push((segs[0].trim().to_string(), segs[1].trim().to_string()));
                }
            }
        },
    }
    
    let mut h = if values.len() == 0 {
        HttpSender::new(server.as_slice(), page.as_slice())
    } else {
        HttpSender::create_request(server.as_slice(), page.as_slice(), "POST").add_arguments(values)
    };
    match h.send_request() {
        Err(e) => println!("Error: {}", e),
        Ok(_) => match h.get_response() {
            Err(e) => println!("Error: {}", e),
            Ok(response) => {
                println!("Response from server:");
                println!("HTTP/{} {} {}\n", response.version, response.reason, response.status);
                for (v, k) in response.headers.iter() {
                    println!("{} {}", v, k);
                }
                println!("\n{}", response.body)
            },
        },
    }
}