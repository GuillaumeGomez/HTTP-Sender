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
use collections::HashMap;
use gzip_reader::GzipReader;
use std::ascii::OwnedStrAsciiExt;
use std::num::from_str_radix;
use std::strbuf::StrBuf;

mod gzip_reader;

struct ChunkReader {
    data: Vec<u8>,
}

impl ChunkReader {
    fn new(v : &Vec<u8>) -> ChunkReader {
        ChunkReader{data: v.clone()}
    }
    fn readFromChunk(&self) -> Result<(uint, uint), ~str> {
        let mut line = StrBuf::new();
        static MAXNUM_SIZE : uint = 16;
        static HEX_CHARS : &'static [u8] = bytes!("0123456789abcdefABCDEF");
        let mut is_in_chunk_extension = false;
        let mut pos = 0;

        if self.data.len() > 1 && self.data.get(0) == &0u8 && self.data.get(1) == &0u8 {
            return Ok((0, self.data.len() - 1));
        }
        while pos < self.data.len() {
            match *self.data.get(pos) as char {
                '\r' => { // '\r'
                    pos += 1;
                    if pos >= self.data.len() || self.data.get(pos) != &0x0au8 {
                        return Err("Error with '\r'".to_owned());
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
                    return Err("Chunk format error".to_owned())}
                ,
            }
        }

        if line.len() > MAXNUM_SIZE {
            return Err("http chunk transfer encoding format: size line too long".to_owned());
        }
        match from_str_radix(line.as_slice(), 16) {
            Some(v) => Ok((v, pos + 1)),
            None => Ok((0, 0)),
        }
    }

    fn readNext(&mut self) -> Result<(Vec<u8>, uint), ~str> {
        let mut out = Vec::new();

        if self.data.len() > 0 {
            match self.readFromChunk() {
                Ok((toWrite, toSkip)) => {
                    if toWrite == 0 {
                        return Ok((out.clone(), 0));
                    }
                    let mut tmp_v = self.data.clone().move_iter().skip(toSkip).collect::<Vec<u8>>();

                    tmp_v.truncate(toWrite);

                    out.push_all_move(tmp_v);
                    self.data = self.data.clone().move_iter().skip(toSkip + toWrite).collect::<Vec<u8>>();
                    Ok((out.clone(), self.data.len()))
                }
                Err(e) => Err(e),
            }
        } else {
            Ok((out.clone(), 0))
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

    fn fromGzip(&self, v: Vec<u8>) -> Result<~str, ~str> {
        let mut r = GzipReader{inner: v};
        match r.decode() {
            Ok(res) => Ok(res),
            Err(res) => Err(res.to_owned()),
        }
    }

    fn fromUtf8(&self, v: Vec<u8>) -> Result<~str, ~str> {
        match StrBuf::from_utf8(v) {
            None => Err("Couldn't convert body to UTF-8".to_owned()),
            Some(tmp) => Ok(tmp.to_str()),
        }
    }

    fn readAllChunked(&self, mut cr : ChunkReader, gzip : bool, mut out : StrBuf) -> Result<~str, ~str> {
        match cr.readNext() {
            Err(res) => Err(res.to_owned()),
            Ok((res, size)) => {
                if size > 0 {
                    match if gzip == true {
                        self.fromGzip(res)
                    } else {
                        self.fromUtf8(res)
                    } {
                        Ok(s) => {
                            out.push_str(s);
                            self.readAllChunked(cr, gzip, out)
                        }
                        Err(e) => Err(e.to_owned()),
                    }
                } else {
                    Ok(out.into_owned())
                }
            }
        }
    }

    fn fromChunked(&self, v: Vec<u8>, gzip: bool) -> Result<~str, ~str> {
        let cr = ChunkReader::new(&v);
        let out = StrBuf::new();

        self.readAllChunked(cr, gzip, out)
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
        let mut chunked = false;
        for (v, k) in headers.iter() {
            let tmp_s = v.clone().into_ascii_lower();
            if tmp_s == "content-encoding".to_owned() {
                if k.contains(&("gzip".to_owned())) {
                    gzip = true;
                }
            } else if tmp_s == "transfer-encoding".to_owned() {
                if k.contains(&("chunked".to_owned())) {
                    chunked = true;
                }
            }
        }

        match stream.read_to_end() {
            Err(_) => return Err("Couldn't read body".to_owned()),
            Ok(l) => {
                match if chunked == true {
                    self.fromChunked(l, gzip)
                } else if gzip == true {
                    self.fromGzip(l)
                } else {
                    self.fromUtf8(l)
                } {
                   Err(e) => Err(e),
                   Ok(r) => Ok(ResponseData{body: r.into_owned(), headers: headers,
                            version: version.to_owned(), status: status.to_owned(), reason: reason.to_owned()}),
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