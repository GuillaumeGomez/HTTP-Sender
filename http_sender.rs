#![crate_id = "http_sender#0.1"]
#![crate_type = "lib"]

#![allow(dead_code)]

extern crate collections;
extern crate libc;

use std::io::TcpStream;
use std::io::TcpStream::connect;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::SocketAddr;
use std::io::BufferedReader;
use std::io::IoResult;
use collections::HashMap;
use gzip_reader::GzipReader;
use std::slice;

mod gzip_reader;

struct MyReader {
    data: Vec<u8>,
    start: uint,
}

impl Reader for MyReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        println!("size: {} / {}", buf.len(), self.data.len());
        let tmp = if self.data.len() - self.start > buf.len() {
                    println!("if -> {}", buf.len())
                    buf.len()
                } else {
                    println!("else -> {}", self.data.len() - self.start)
                    self.data.len() - self.start
                };
        if tmp == 0 {
            Ok(0)
        } else {
            slice::bytes::copy_memory(buf, self.data.slice(self.start, tmp));
            self.start = tmp;
            Ok(tmp)
        }
    }
}

pub struct ResponseData {
    pub headers: HashMap<~str, Vec<~str>>,
    pub version: ~str,
    pub status: ~str,
    pub reason: ~str,
    pub body: ~str,
}

pub struct HttpSender {
    address: ~str,
    page: ~str,
    port: u16,
    socket: Option<TcpStream>,
}

impl HttpSender {
    pub fn new(server_address: &str, page: &str) -> HttpSender {
        HttpSender{address: server_address.to_owned(), page: page.to_owned(), port: 80, socket: None}
    }

    fn createHeader(&self) -> ~str {
        format!("GET {} HTTP/1.1\r\n\
                Host: {}\r\n\
                Accept: text/plain,text/html,application/rss+xml,*/*\r\n\
                Accept-Language: fr,en-US;q=0.8,en;q=0.6\r\n\
                Accept-Encoding: gzip,deflate\r\n\
                connection: close\r\n\
                User-Agent: Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)\r\n\r\n", self.page, self.address)
    }

    pub fn getResponse(&self) -> Result<ResponseData, ~str> {
        let mut stream;

        match self.socket {
            None => return Err("Not connected to server".to_owned()),
            Some(ref e) => stream = BufferedReader::with_capacity(1, e.clone()),
        }
        let response;

        match stream.read_line() {
            Err(_) => return Err("read line failed when getting data".to_owned()),
            Ok(l) => response = l,
        }

        let segs = response.splitn(' ', 2).collect::<Vec<&str>>();

        let version;
        match *segs.get(0) {
            "HTTP/1.1" => version = "1.1",
            "HTTP/1.0" => version = "1.0",
            v if v.starts_with("HTTP/") => version = "1.0",
            _ => return Err("Unsupported HTTP version".to_owned()),
        };

        let status = segs.get(1);
        let reason = segs.get(2).trim_right();

        let mut headers = HashMap::new();

        loop {
            let line;

            match stream.read_line() {
                Err(_) => return Err("Errow while reading header".to_owned()),
                Ok(l) => line = l,
            }

            let segs = line.splitn(':', 1).collect::<Vec<&str>>();
            if segs.len() == 2 {
                let k = segs.get(0).trim();
                let v = segs.get(1).trim();
                headers.insert_or_update_with(k.to_owned(), vec!(v.into_owned()), |_k, ov| ov.push(v.into_owned()));
            } else {
                if [~"\r\n", ~"\n", ~""].contains(&line) {
                    break;
                }
                return Err(line.to_owned());
            }
        }
        
        let mut gzip = false;
        for (v, k) in headers.iter() {
            println!("{}: {}", v, k);
        }
        for (v, k) in headers.iter() {
            if v == &("Content-Encoding".to_owned()) {
                if k.contains(&("gzip".to_owned())) {
                    gzip = true;
                }
                break;
            }
        }

        match stream.read_to_end() {
            Err(_) => return Err("Couldn't read body".to_owned()),
            Ok(l) => {
                if gzip == true {
                    let mut r = GzipReader{inner: l};
                    match r.decode() {
                        Ok(res) => Ok(ResponseData{body: res.to_owned(), headers: headers,
                                    version: version.to_owned(), status: status.to_owned(), reason: reason.to_owned()}),
                        Err(res) => Err(res.to_owned()),
                    }
                } else {
                    match StrBuf::from_utf8(l) {
                        None => Err("Couldn't convert body to UTF-8".to_owned()),
                        Some(tmp) => Ok(ResponseData{body: tmp.into_owned(), headers: headers,
                                        version: version.to_owned(), status: status.to_owned(), reason: reason.to_owned()}),
                    }
                }
            },
        }
    }

    pub fn sendRequest(& mut self) -> Result<(), ~str> {
        let addr;
        match get_host_addresses(self.address) {
            Err(_) => return Err("Couldn't find host address".to_owned()),
            Ok(ret) => addr = ret,
        };

        addr.iter().skip_while(|&a| {
            self.socket = TcpStream::connect(SocketAddr{ip: *a, port: 80}).ok(); self.socket.is_none()}).next();
        if self.socket.is_some() {
            let t = self.createHeader();
            match self.socket.get_mut_ref().write(t.into_bytes()) {
                Err(_) => Err("Couldn't send message".to_owned()),
                Ok(_) => Ok(()),
            }
        } else {
            Err("Couldn't send message".to_owned())
        }
    }
}