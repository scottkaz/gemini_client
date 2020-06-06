use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::env;
use url::{Url, ParseError};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let input_url = "gemini://192.168.0.106:1965/client_test.gmi";
    let url = Url::parse(input_url).unwrap();
    println!("{}", url.scheme());
    println!("{}", url.host_str().unwrap());
    println!("{}", url.host().unwrap());
    println!("{}", url.port().unwrap_or(0));
    println!("{}", url.path());
    let socket_addr = url.socket_addrs(|| Some(1965)).unwrap();

    let connector = TlsConnector::builder()
        .use_sni(false)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream = TcpStream::connect(&*socket_addr).unwrap();
    let mut stream = connector.connect("", stream).unwrap();

    let mut request = String::from(input_url);
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    let instring = String::from_utf8_lossy(&res);

    let response_header = gemini_client::get_response_header(&instring);
    println!("{:?}", response_header);
    let response_body = gemini_client::get_response_body(&instring);
    println!("{}", response_body);
}

