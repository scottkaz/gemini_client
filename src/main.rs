use native_tls::TlsConnector;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = gemini_client::Config::new(&args).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let connector = TlsConnector::builder()
        .use_sni(false)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream = TcpStream::connect(&*config.socket_addr).unwrap();
    let mut stream = connector.connect("", stream).unwrap();

    let mut request = config.input_url.to_string();
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    let instring = String::from_utf8_lossy(&res);

    let response_header = gemini_client::get_response_header(&instring);
    println!("{:?}", response_header);
    let response_body = gemini_client::get_response_body(&instring);
    println!("{}", response_body);
}
