extern crate collections;

use std::io::TcpStream;
use std::io::TcpStream::connect;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::SocketAddr;
use std::io::BufferedReader;
use collections::HashMap;
use std::os;

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
                                Accept: text/plain,text/html,*/*\r\n\
                                Accept-Language: fr,en-US;q=0.8,en;q=0.6\r\n\
                                Accept-Encoding: identity\r\n\
                                connection: close\r\n\
                                User-Agent: Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)\r\n\r\n", self.page, self.address);
                match sock.write(t.into_bytes()) {
                    Err(_) => fail!("Couldn't send message"),
                    Ok(_) => {
                        let mut stream = BufferedReader::with_capacity(1, sock.clone());
                        let response = stream.read_line().ok().expect("read line failed when getting data");

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
                            let line = stream.read_line().ok().expect("read line failed in headers read");

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
                        let tmp = StrBuf::from_utf8(stream.read_to_end().ok().expect("read to end failed")).expect("from utf8 failed");

                        ResponseData{body: tmp.into_owned(), headers: headers,
                            version: version.to_owned(), status: status.to_owned(), reason: reason.to_owned()}
                    }
                }
            },
        }
    }
}

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
    
    let h = HttpSender{address: server, page: page, port: 80};
    let response = h.sendRequest("");

    println!("Response from server:");
    println!("HTTP/{} {} {}\n", response.version, response.reason, response.status);
    for (v, k) in response.headers.iter() {
        println!("{}: {}", v, k);
    }
    println!("\n{}", response.body);
}