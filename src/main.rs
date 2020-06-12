use std::env;
use gemini_client::{get_tls_stream, send_request, Header, Status, BodyData};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = gemini_client::Config::new(&args).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let mut stream = get_tls_stream(&config.socket_addr);

    let instring = send_request(&mut stream, &config.input_url);

    let response_header = gemini_client::get_response_header(&instring)
        .expect("problem with response header");
    println!("{:?}", response_header);

    match response_header.get_status() {
        Some(Status::Success) => {
            let response_body =
                gemini_client::get_response_body(&response_header.meta, &instring);
            match response_body {
                Ok(BodyData::Text(s)) => println!("{}", s),
                Ok(BodyData::Binary(s)) => println!("{:?}", s),
                Err(e) => println!("{}", e),
            }
        },
        _ => println!("status code {} not implemented", response_header.status_code),
    }
}
