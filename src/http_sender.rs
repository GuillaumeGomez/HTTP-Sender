#![crate_name = "http_sender"]
#![crate_type = "lib"]

#![allow(dead_code)]
#![allow(improper_ctypes)]
#![feature(globs)]

extern crate collections;
extern crate libc;
extern crate time;

use std::io::TcpStream;
use std::io::net::addrinfo::get_host_addresses;
use std::io::BufferedReader;
use std::collections::HashMap;
use gzip_reader::GzipReader;
use std::num::from_str_radix;
use std::io;
use std::io::{File, Open, Write};
use time::get_time;
use std::collections::hash_map::{Occupied, Vacant};
use std::ascii::OwnedAsciiExt;

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
                    line.push(c);
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
                        let mut tmp_v = self.data.clone().into_iter().skip(to_skip).collect::<Vec<u8>>();

                        tmp_v.truncate(to_write);
                        out.extend(tmp_v.into_iter());
                        self.data = self.data.clone().into_iter().skip(to_skip + to_write).collect::<Vec<u8>>();
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
    verbose: bool,
    output_file: Option<String>
}

impl HttpSender {
    // same as HttpSender::create_request(address, page, "GET")
    pub fn new(server_address: &str, page: &str) -> HttpSender {
        HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
            args: HashMap::new(), request_type: "GET".to_string(), user_agent: "imperio-test/0.1".to_string(),
            verbose: false, output_file: None}
    }

    pub fn create_request(server_address : &str, page: &str, request_method : &str) -> HttpSender {
        let requests = ["GET", "HEAD", "PUT", "POST"];

        if requests.contains(&request_method.clone()) {
            HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
                        args: HashMap::new(), request_type: request_method.to_string(),
                        user_agent: "imperio-test/0.1".to_string(), verbose: false, output_file: None}
        } else {
            HttpSender{address: server_address.to_string(), page: page.to_string(), port: 80, socket: None,
                        args: HashMap::new(), request_type: "GET".to_string(),
                        user_agent: "imperio-test/0.1".to_string(), verbose: false, output_file: None}
        }
    }

    pub fn set_output_filename(&mut self, file_name: &str) {
        self.output_file = Some(String::from_str(file_name));
    }

    pub fn get_output_filename(&self) -> Option<String> {
        self.output_file.clone()
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn add_argument(mut self, key: &str, value: &str) -> HttpSender {
        let c_v = value.to_string();

        match self.args.entry(key.to_string()) {
            Vacant(entry) => entry.set(vec!(c_v.to_string())),
            Occupied(mut entry) => {
                (*entry.get_mut()).push(c_v.to_string());
                entry.into_mut()
            }
        };
        self
    }

    pub fn add_arguments(mut self, arguments: Vec<(String, String)>) -> HttpSender {
        for &(ref k, ref v) in arguments.iter() {
            let t_k = k.clone();
            let c_v = v.to_string();

            match self.args.entry(t_k.to_string()) {
                Vacant(entry) => entry.set(vec!(v.to_string())),
                Occupied(mut entry) => {
                    (*entry.get_mut()).push(c_v.to_string());
                    entry.into_mut()
                }
            };
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
            
            if self.verbose {
                println!("req : {}", tmp);
            }
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
        match String::from_utf8(v.clone()) {
            Err(_) => Ok(String::from_utf8_lossy(v.as_slice()).to_string()),
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
        let segs = response.as_slice().splitn(2, ' ').collect::<Vec<&str>>();
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

            let segs = line.as_slice().splitn(1, ' ').collect::<Vec<&str>>();
            if segs.len() == 2 {
                let k = segs[0].trim();
                let v = segs[1].trim();

                match headers.entry(k.to_string()) {
                    Vacant(entry) => entry.set(vec!(v.to_string())),
                    Occupied(mut entry) => {
                        (*entry.get_mut()).push(v.to_string());
                        entry.into_mut()
                    }
                };
            } else {
                if ["\r\n".to_string(), "\n".to_string(), "".to_string()].contains(&line) {
                    break;
                }
                return Err(line.to_string());
            }
        }
        
        let mut gzip = false;
        let mut chunked = false;

        if is_byte_response(&headers) {
            return get_bytes_data(&headers, &mut stream, version, status, reason,
                match self.output_file {
                    None => "".into_string(),
                    Some(ref f) => f.clone()
                }, self.verbose);
        }
        for (v, k) in headers.iter() {
            let tmp_s = v.clone().into_ascii_lower();
            if tmp_s == "content-encoding:".to_string() {
                if k.contains(&("gzip".to_string())) {
                    gzip = true;
                }
            } else if tmp_s == "transfer-encoding:".to_string() {
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

            if self.verbose {
                println!("ip: {}", s_ip);
            }
            match TcpStream::connect(format!("{}:{}", s_ip, 80u).as_slice()) {
                Ok(s) => {
                    self.socket = Some(s);
                    false
                }
                Err(_) => {
                    self.socket = None;
                    true
                }
            }
        }).next();
        if self.socket.is_some() {
            let t = self.create_header();
            match self.socket.as_mut().unwrap().write(t.into_bytes().as_slice()) {
                Err(_) => Err("Couldn't send message".to_string()),
                Ok(_) => Ok(()),
            }
        } else {
            Err("Couldn't send message".to_string())
        }
    }
}

fn is_byte_response(headers: &HashMap<String, Vec<String>>) -> bool {
    let mut found_range = false;
    let mut found_connection = false;

    for (v, k) in headers.iter() {
        let tmp_s = v.clone().into_ascii_lower();
        if tmp_s == "accept-ranges:".to_string() {
            if k.contains(&("bytes".to_string())) {
                found_range = true;
            }
        } else if tmp_s == "connection:".to_string() {
            if k.contains(&("keep-alive".to_string())) {
                found_connection = true;
            }
        } else if tmp_s == "content-type:".to_string() {
            for it in k.iter() {
                let sub_it = it.as_slice();

                if sub_it.contains("video") || sub_it.contains("audio") {
                    return true;
                }
            }
        }
    }
    found_connection && found_range
}

fn get_file_name() -> String {
    let mut reader = io::stdin();

    print!("Please enter the file name of the content : ");
    reader.read_line().ok().unwrap_or("".to_string())
}

fn clean_useless_bytes(stream: &mut BufferedReader<TcpStream>, begin_bytes: uint, verbose: bool) {
    if begin_bytes > 0 {
        if verbose {
            println!("Let's clean {} byte(s)", begin_bytes);
        }
        let mut read = 0u;
        let mut v = Vec::with_capacity(begin_bytes);

        loop {
            match stream.read_at_least(begin_bytes - read, v.as_mut_slice()) {
                Ok(s) => {
                    read += s;
                }
                Err(e) => {
                    if e.kind == std::io::EndOfFile {
                        panic!("Error with bytes range")
                    } else if e.kind == std::io::NoProgress {
                        
                    } else {
                        panic!("An error occured : {}", e)
                    }
                }
            }
            if read >= begin_bytes {
                break;
            }
        }
    }
}

#[allow(unused_must_use)]
fn get_bytes_data(headers: &HashMap<String, Vec<String>>, stream: &mut BufferedReader<TcpStream>, version: &str, status: &str,
    reason: &str, filename: String, verbose: bool) -> Result<ResponseData, String> {
    let file_name = if filename.as_slice() != "" {
        filename
    } else {
        let mut tmp = get_file_name();
        tmp.pop();
        tmp
    };

    if file_name.as_slice() == "" {
        panic!("invalid file_name");
    }
    let mut file = match File::open_mode(&Path::new(file_name), Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("file error: {}", e),
    };
    let mut length = 0u;
    let mut position = 0u;
    let mut begin_bytes = 0u;

    for (v, k) in headers.iter() {
        let tmp_s = v.clone().into_ascii_lower();
        if tmp_s == "content-length:".to_string() {
            length = from_str(k[0].as_slice()).unwrap();
        } else if tmp_s == "content-range:".to_string() {
            let tmp_begin : Vec<&str> = k[0].as_slice().split_str(" ").collect();
            let begin = tmp_begin[0];
            let tmp_bytes : Vec<&str> = begin.split_str("-").collect();

            begin_bytes = from_str(tmp_bytes[0]).unwrap();
        }
    }
    clean_useless_bytes(stream, begin_bytes, verbose);
    let mut buf = [0, ..100000];
    let mut timer = get_time().sec;
    let mut downloaded_data = 0u;

    loop {
        if buf.len() <= length - position {
            match stream.read(buf) {
                Ok(s) => {
                    let tmp = unsafe { ::std::vec::raw::from_buf(buf.as_ptr(), s) };
                    position += s;
                    file.write(tmp.as_slice());
                    downloaded_data += s;
                }
                Err(e) => {
                    if e.kind == std::io::EndOfFile {
                        break
                    } else {
                        panic!("An error occured : {}", e)
                    }
                }
            }
        } else {
            let mut second_buff = Vec::new();
            match stream.push_at_least(length - position, length - position, &mut second_buff) {
                Ok(s) => {
                    let tmp = unsafe { ::std::vec::raw::from_buf(second_buff.as_ptr(), s) };
                    position += s;
                    file.write(tmp.as_slice());
                    downloaded_data += s;
                }
                Err(e) => {
                    if e.kind == std::io::EndOfFile {
                        break;
                    } else if e.kind == std::io::NoProgress {
                        println!("No progress made...");
                    } else {
                        panic!("An error occured : {}", e);
                    }
                }
            }
        }
        print_stats(length, &mut downloaded_data, position, &mut timer);
        if length <= position {
            break;
        }
    }
    Ok(ResponseData{body: "".into_string(), headers: headers.clone(),
        version: version.to_string(), status: status.to_string(), reason: reason.to_string()})
}

fn print_stats(length: uint, downloaded_data: &mut uint, position: uint, timer: &mut i64) {
    let current_time = get_time().sec;

    if current_time != *timer {
        let remaining = if *downloaded_data > 0 {
            (length - position) / *downloaded_data
        } else {
            0u
        };
        print!("{} / {} -> {}% - remaining time: {}  | {}         \r", position, length,
            position as f32 / length as f32 * 100f32,
            if remaining < 3600 {
                format!("{:02u}:{:02u}", remaining / 60, remaining % 60)
            } else {
                format!("{:02u}:{:02u}:{:02u}", remaining / 3600, remaining / 60, remaining % 60)
            },
            if *downloaded_data < 1000 {
                format!("{} o/s", *downloaded_data)
            } else if *downloaded_data < 1000000 {
                format!("{} Ko/s", *downloaded_data / 1000)
            } else {
                format!("{} Mo/s", *downloaded_data / 1000000)
            });
        io::stdio::flush();
        *timer = current_time;
        *downloaded_data = 0u;
    }
}