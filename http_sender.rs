use std::io::TcpStream;
use std::io::TcpStream::connect;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::SocketAddr;

struct HttpSender {
	address: ~str,
	page: ~str,
	port: u16,
}

impl HttpSender {
	fn sendRequest(&self, message : &str) {
		
		let addr = match get_host_addresses(self.address) {
			Err(_) => fail!("Couldn't find host address"),
        	Ok(ret) => ret
        };

        let mut done = 0;
        for tmp in addr.iter() {
        	println!("-> {}", tmp);
        	match TcpStream::connect(SocketAddr{ip: *tmp, port: self.port}) {
        		Err(_) => {},
        		Ok(ret) => {
        			let mut socket = ret;
        			let mut t = format!("GET {} HTTP/1.1", self.page);

					t = format!("{}\r\nHost: {}\r\n", t, self.address);
					t = format!("{}User-Agent: Mozilla/5.0 (Windows; U; Windows NT 6.1; en-US; rv:1.9.1.5) Gecko/20091102 Firefox/3.5.5 (.NET CLR 3.5.30729)\r\n", t);
					t = format!("{}Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n", t);
					t = format!("{}Accept-Language: en-us,en;q=0.5\r\n", t);
					t = format!("{}Accept-Encoding:\r\n", t);
					done = 1;

			    	match socket.write(t.into_bytes()) {
			        	Err(_) => fail!("Couldn't send message"),
			        	Ok(_) => {
			        		let response = socket.read_to_end();
			        		println!("Response from server :\n{}", response);
			        	}
			    	}
        			break;
        		},
        	}
        }
        if done == 0 {
        	fail!("Couldn't connect to host");
    	}
	}
}

/*Accept-Encoding: gzip,deflate
Accept-Charset: ISO-8859-1,utf-8;q=0.7,*;q=0.7
Keep-Alive: 300
Connection: keep-alive
Cookie: PHPSESSID=r2t5uvjq435r4q7ib3vtdjq120
Pragma: no-cache
Cache-Control: no-cache*/

fn main() {
	println!("bonjour !");
	let h = HttpSender{address: "www.google.fr".to_owned(), page: "".to_owned(), port: 80};
	h.sendRequest("boudoum");
}