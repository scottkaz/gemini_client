use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let connector = TlsConnector::builder()
        .use_sni(false)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream = TcpStream::connect("192.168.0.106:1965").unwrap();
    let mut stream = connector.connect("", stream).unwrap();
    // let mut stream = connector.connect("google.com", stream).unwrap();

    stream
        .write_all(b"gemini://192.168.0.106:1965/\r\n")
        .unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    let instring = String::from_utf8_lossy(&res);

    let response_header = gemini_client::get_response_header(&instring);
    println!("{:?}", response_header);
}

