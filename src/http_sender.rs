#![crate_name = "http_sender"]
#![crate_type = "lib"]

#![allow(dead_code)]
#![feature(globs)]

extern crate collections;
extern crate libc;

use std::io::net::tcp::TcpStream;
use std::io::net::addrinfo::get_host_addresses;
use std::io::BufferedReader;
use std::collections::HashMap;
use gzip_reader::GzipReader;
use std::ascii::OwnedStrAsciiExt;
use std::num::from_str_radix;

mod gzip_reader;
mod zlib;

struct ChunkReader {
    data: Vec<u8>,
}

impl ChunkReader {
    fn new(v : &Vec<u8>) -> ChunkReader {
        ChunkReader{data: v.clone()}
    }

    fn read_from_chunk(&self) -> Result<(uint, uint), String> {
        let mut line = String::new();
        static MAXNUM_SIZE : uint = 16;
        static HEX_CHARS : &'static [u8] = b"0123456789abcdefABCDEF";
        let mut is_in_chunk_extension = false;
        let mut pos = 0;

        if self.data.len() > 1 && self.data[0] == 0u8 && self.data[1] == 0u8 {
            return Ok((0, self.data.len() - 1));
        }
        while pos < self.data.len() {
            match self.data[pos] as char {
                '\r' => {
                    pos += 1;
                    if pos >= self.data.len() || self.data[pos] != 0x0au8 {
                        return Err("Error with '\r'".to_string());
                    }
                    break;
                }
                '\n' => break,
                _ if is_in_chunk_extension => {
                    pos += 1;
                    continue;
                }
                ';' => {is_in_chunk_extension = true;}
                c if HEX_CHARS.contains(&(c as u8)) => {
                    line.push_char(c);
                    pos += 1;
                }
                _ => {
                    println!("{}", self.data);
                    return Err("Chunk format error".to_string())
                }
            }
        }

        if line.len() > MAXNUM_SIZE {
            Err("http chunk transfer encoding format: size line too long".to_string())
        } else {
            match from_str_radix(line.as_slice(), 16) {
                Some(v) => Ok((v, pos + 1)),
                None => Ok((0, 0)),
            }
        }
    }

    fn read_next(&mut self) -> Result<(Vec<u8>, uint), String> {
        let mut out = Vec::new();

        if self.data.len() > 0 {
            match self.read_from_chunk() {
                Ok((to_write, to_skip)) => {
                    if to_write == 0 {
                        Ok((out.clone(), 0))
                    } else {
                        let mut tmp_v = self.data.clone().move_iter().skip(to_skip).collect::<Vec<u8>>();

                        tmp_v.truncate(to_write);
                        out.push_all_move(tmp_v);
                        self.data = self.data.clone().move_iter().skip(to_skip + to_write).collect::<Vec<u8>>();
                        Ok((out.clone(), self.data.len()))
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            Ok((out.clone(), 0))
        }
    }
}

pub struct ResponseData {
    pub headers: HashMap<String, Vec<String>>,
    pub version: String,
    pub status: String,
    pub reason: String,
    pub body: String,
}

pub struct HttpSender {
    address: String,
    page: String,
    port: u16,
    socket: Option<TcpStream>,
    args: HashMap<String, Vec<String>>,
    request_type: String,
    user_agent: String,
}

impl HttpSender {
    // same as HttpSender::create_request(address, page, "GET")
    pub fn new(server_address: &str, page: &str) -> HttpSender {
        HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
            args: HashMap::new(), request_type: "GET".to_string(), user_agent: "imperio-test/0.1".to_string()}
    }

    pub fn create_request(server_address : &str, page: &str, request_method : &str) -> HttpSender {
        let requests = ["GET", "HEAD", "PUT", "POST"];

        if requests.contains(&request_method.clone()) {
            HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
                        args: HashMap::new(), request_type: request_method.to_string(),
                        user_agent: "imperio-test/0.1".to_string()}
        } else {
            HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
                        args: HashMap::new(), request_type: "GET".to_string(),
                        user_agent: "imperio-test/0.1".to_string()}
        }
    }

    pub fn add_argument(mut self, key: &str, value: &str) -> HttpSender {
        let c_v = value.to_string();
        self.args.insert_or_update_with(key.to_string(),
                                        vec!(value.to_string()),
                                        |ref _k, v| v.push(c_v.to_string()));
        self
    }

    pub fn add_arguments(mut self, arguments: Vec<(String, String)>) -> HttpSender {
        for &(ref k, ref v) in arguments.iter() {
            let t_k = k.clone();
            let c_v = v.to_string();
            self.args.insert_or_update_with(t_k.to_string(),
                                        vec!(v.to_string()),
                                        |ref _k, v| v.push(c_v.to_string()));
        }
        self
    }

    fn create_simple_header(&self, args : String) -> String {
        let mut t_args = args.clone();

        if t_args.len() > 0 {
            let tmp = t_args.into_string();

            t_args = String::new();
            t_args.push_str("?");
            t_args.push_str(tmp.as_slice());
        }

        format!("{} {}{} HTTP/1.1\r\n\
                Host: {}\r\n\
                Accept: text/plain,text/html,application/rss+xml,*/*\r\n\
                Accept-Language: fr,en-US;q=0.8,en;q=0.6\r\n\
                Accept-Encoding: gzip,deflate\r\n\
                connection: close\r\n\
                User-Agent: {}\r\n\r\n", self.request_type, self.page, t_args.into_string(), self.address, self.user_agent)
    }

    fn create_header_with_args(&self, args : String) -> String {
        format!("{} {} HTTP/1.1\r\n\
                Host: {}\r\n\
                Accept: text/plain,text/html,application/rss+xml,*/*\r\n\
                Accept-Language: fr,en-US;q=0.8,en;q=0.6\r\n\
                Accept-Encoding: gzip,deflate\r\n\
                connection: close\r\n\
                User-Agent: {}\r\n\
                Content-Type: application/x-www-form-urlencoded\r\n\
                Content-Length: {}\r\n\r\n{}\r\n", self.request_type, self.page, self.address, self.user_agent, args.len(), args.into_string())
    }

    fn create_header(&self) -> String {
        let mut args = String::new();

        for (tmp, v) in self.args.iter() {
            for in_tmp in v.iter() {
                if args.len() > 0 {
                    args.push_str("&");
                }
                args.push_str(tmp.as_slice());
                args.push_str("=");
                args.push_str(in_tmp.as_slice());
            }
        }

        if self.request_type == "GET".to_string() || self.request_type == "HEAD".to_string() {
            let tmp = self.create_simple_header(args.clone());
            println!("req : {}", tmp);
            tmp
        } else {
            self.create_header_with_args(args.clone())
        }
    }

    fn from_gzip(&self, v: Vec<u8>) -> Result<String, String> {
        let mut g = GzipReader{inner: v};

        match g.decode() {
            Ok(res) => Ok(res),
            Err(res) => Err(res.to_string()),
        }
    }

    fn from_utf8(&self, v: Vec<u8>) -> Result<String, String> {
        match String::from_utf8(v) {
            Err(_) => Err(String::from_str("Couldn't convert body to UTF-8")),
            Ok(tmp) => Ok(tmp),
        }
    }

    fn read_all_chunked(&self, mut cr : ChunkReader, gzip : bool, mut out : String) -> Result<String, String> {
        match cr.read_next() {
            Err(res) => Err(res.to_string()),
            Ok((res, size)) => {
                if size > 0 {
                    match if gzip == true {
                        self.from_gzip(res)
                    } else {
                        self.from_utf8(res)
                    } {
                        Ok(s) => {
                            out.push_str(s.as_slice());
                            self.read_all_chunked(cr, gzip, out)
                        }
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    Ok(out.into_string())
                }
            }
        }
    }

    fn from_chunked(&self, v: Vec<u8>, gzip: bool) -> Result<String, String> {
        self.read_all_chunked(ChunkReader::new(&v), gzip, String::new())
    }

    pub fn get_response(&self) -> Result<ResponseData, String> {
        let mut stream = match self.socket {
            None => return Err("Not connected to server".to_string()),
            Some(ref e) => BufferedReader::with_capacity(1, e.clone()),
        };
        let response = match stream.read_line() {
            Err(_) => return Err("read line failed when getting data".to_string()),
            Ok(l) => l,
        };
        let segs = response.as_slice().splitn(' ', 2).collect::<Vec<&str>>();
        let version = match segs[0] {
            "HTTP/1.1" => "1.1",
            "HTTP/1.0" => "1.0",
            v if v.starts_with("HTTP/") => "1.0",
            _ => return Err("Unsupported HTTP version".to_string()),
        };

        let status = segs[1];
        let reason = segs[2].trim_right();

        let mut headers = HashMap::new();

        loop {
            let line = match stream.read_line() {
                Err(_) => return Err("Errow while reading header".to_string()),
                Ok(l) => l,
            };

            let segs = line.as_slice().splitn(':', 1).collect::<Vec<&str>>();
            if segs.len() == 2 {
                let k = segs[0].trim();
                let v = segs[1].trim();
                headers.insert_or_update_with(k.to_string(), vec!(v.into_string()), |_k, ov| ov.push(v.into_string()));
            } else {
                if ["\r\n".to_string(), "\n".to_string(), "".to_string()].contains(&line) {
                    break;
                }
                return Err(line.to_string());
            }
        }
        
        let mut gzip = false;
        let mut chunked = false;
        for (v, k) in headers.iter() {
            let tmp_s = v.clone().into_ascii_lower();
            if tmp_s == "content-encoding".to_string() {
                if k.contains(&("gzip".to_string())) {
                    gzip = true;
                }
            } else if tmp_s == "transfer-encoding".to_string() {
                if k.contains(&("chunked".to_string())) {
                    chunked = true;
                }
            }
        }

        match stream.read_to_end() {
            Err(_) => return Err("Couldn't read body".to_string()),
            Ok(l) => {
                match if chunked == true {
                    self.from_chunked(l, gzip)
                } else if gzip == true {
                    self.from_gzip(l)
                } else {
                    self.from_utf8(l)
                } {
                   Err(e) => Err(e),
                   Ok(r) => Ok(ResponseData{body: r.into_string(), headers: headers,
                            version: version.to_string(), status: status.to_string(), reason: reason.to_string()}),
                }
            },
        }
    }

    pub fn send_request(& mut self) -> Result<(), String> {
        let addr = match get_host_addresses(self.address.as_slice()) {
            Err(_) => return Err("Couldn't find host address".to_string()),
            Ok(ret) => ret,
        };

        addr.iter().skip_while(|&a| {
            let s_ip = format!("{}", *a);

            println!("ip: {}", s_ip);
            self.socket = TcpStream::connect(s_ip.as_slice(), 80).ok(); self.socket.is_none()}).next();
        if self.socket.is_some() {
            let t = self.create_header();
            match self.socket.get_mut_ref().write(t.into_bytes().as_slice()) {
                Err(_) => Err("Couldn't send message".to_string()),
                Ok(_) => Ok(()),
            }
        } else {
            Err("Couldn't send message".to_string())
        }
    }
}