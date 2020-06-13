use gemini_client::{get_tls_stream, send_request, BodyData, Config, Header, Status};
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let config = gemini_client::Config::new().unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let mut stream = get_tls_stream(&config.socket_addr);

    let instring = send_request(&mut stream, &config.input_url);

    let response_header =
        gemini_client::get_response_header(&instring).expect("problem with response header");
    println!("{:?}", response_header);

    match response_header.get_status() {
        Some(Status::Success) => {
            output_body(&response_header, &instring, &config);
        }
        _ => println!(
            "status code {} not implemented",
            response_header.status_code
        ),
    };
}

fn output_body(response_header: &Header, instring: &Vec<u8>, config: &Config) {
    let response_body = gemini_client::get_response_body(&response_header.meta, &instring);
    match response_body {
        Ok(BodyData::Text(s)) => match &config.output_file {
            Some(filename) => {
                File::create(filename)
                    .expect("could not open file")
                    .write_all(s.as_bytes());
            }
            None => println!("{}", s),
        },
        Ok(BodyData::Binary(s)) => match &config.output_file {
            Some(filename) => {
                File::create(filename)
                    .expect("could not open file")
                    .write_all(&s[..]);
            }
            None => println!("<Binary body>"),
        },
        Err(e) => println!("{}", e),
    };
}
