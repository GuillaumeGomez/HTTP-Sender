extern crate collections;

use std::io::TcpStream;
use std::io::TcpStream::connect;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::SocketAddr;
use std::io::BufferedReader;
use collections::HashMap;

struct ResponseData {
    headers: HashMap<~str, Vec<~str>>,
    version: ~str,
    status: ~str,
    reason: ~str,
    body: ~str,
}

struct HttpSender {
    address: ~str,
    page: ~str,
    port: u16,
}

impl HttpSender {
    fn sendRequest(&self, message : &str) -> ResponseData {
        
        let addr = match get_host_addresses(self.address) {
            Err(_) => fail!("Couldn't find host address"),
            Ok(ret) => ret
        };

        let mut socket: Option<TcpStream> = None;
        addr.iter().skip_while(|&a| {
            socket = TcpStream::connect(SocketAddr{ip: *a, port: 80}).ok();
            socket.is_none()}).next();
        match socket {
            None => fail!("Couldn't connect to server"),
            Some(mut sock) => {
                let t = format!("GET {} HTTP/1.1\r\n\
                                Host: {}\r\n\
                                Accept: text/plain,text/html\r\n\
                                Accept-Language: en-us\r\n\
                                Accept-Encoding: identity\r\n\
                                connection: close\r\n\
                                User-Agent: imperio/1.0\r\n\r\n", self.page, self.address);
                match sock.write(t.into_bytes()) {
                    Err(_) => fail!("Couldn't send message"),
                    Ok(_) => {
                        let mut stream = BufferedReader::with_capacity(1, sock.clone());
                        let response = stream.read_line().unwrap();

                        let segs = response.splitn(' ', 2).collect::<Vec<&str>>();

                        let version = match *segs.get(0) {
                            "HTTP/1.1"                  => "1.1",
                            "HTTP/1.0"                  => "1.0",
                            v if v.starts_with("HTTP/") => "1.0",
                            _                           => fail!("unsupported HTTP version")
                        };
                        let status = segs.get(1);
                        let reason = segs.get(2).trim_right();

                        let mut headers = HashMap::new();

                        loop {
                            let line = stream.read_line().unwrap();

                            let segs = line.splitn(':', 1).collect::<Vec<&str>>();
                            if segs.len() == 2 {
                                let k = segs.get(0).trim();
                                let v = segs.get(1).trim();
                                headers.insert_or_update_with(k.to_owned(), vec!(v.into_owned()), |_k, ov| ov.push(v.into_owned()));
                            } else {
                                if [~"\r\n", ~"\n", ~""].contains(&line) {
                                    break;
                                }
                                fail!("error on this line: {}\n", line);
                            }
                        }

                        ResponseData{body: stream.read_to_str().unwrap().to_owned(), headers: headers,
                            version: version.to_owned(), status: status.to_owned(), reason: reason.to_owned()}
                    }
                }
            },
        }
    }
}

fn main() {
    let h = HttpSender{address: "www.guillaume-gomez.fr".to_owned(), page: "/".to_owned(), port: 80};
    let response = h.sendRequest("");

    println!("Response from server:");
    println!("HTTP/{} {} {}\n", response.version, response.reason, response.status);
    for (v, k) in response.headers.iter() {
        println!("{}: {}", v, k);
    }
    println!("\n{}", response.body);
}