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

       	let mut socket: Option<TcpStream> = None;
		addr.iter().skip_while(|&a| {
			socket = TcpStream::connect(SocketAddr{ip: *a, port: 80}).ok();
			socket.is_none()}).next();
		match socket {
			None => fail!("Couldn't connect to server"),
			Some(mut sock) => {
				let mut t = format!("GET {} HTTP/1.1", self.page);

				t = format!("{}\r\nHost: {}\r\n", t, self.address);
				t = format!("{}User-Agent: Mozilla/5.0 (Windows; U; Windows NT 6.1; en-US; rv:1.9.1.5) Gecko/20091102 Firefox/3.5.5 (.NET CLR 3.5.30729)\r\n", t);
				t = format!("{}Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n", t);
				t = format!("{}Accept-Language: en-us,en;q=0.5\r\n", t);
				t = format!("{}Accept-Encoding:\r\n", t);
	        	match sock.write(t.into_bytes()) {
				    Err(_) => fail!("Couldn't send message"),
				    Ok(_) => {
				    	let response = sock.read_to_end();
				        println!("Response from server :\n{}", response);
				    }
				}
			},
		}
	}
}

fn main() {
	println!("bonjour !");
	let h = HttpSender{address: "www.google.fr".to_owned(), page: "".to_owned(), port: 80};
	h.sendRequest("boudoum");
}