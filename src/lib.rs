use clap;
use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use native_tls::{TlsConnector, TlsStream};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::string::FromUtf8Error;
use url::Url;

pub struct Config {
    pub input_url: Url,
    pub socket_addr: Vec<SocketAddr>,
    pub output_file: Option<String>,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn std::error::Error>> {
        let args = App::new("gemini-client")
            .version(crate_version!())
            .author(crate_authors!())
            .about("Simple gemini protocol client")
            .arg(Arg::with_name("URL").help("The URL to fetch").index(1))
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("outfile")
                    .value_name("FILE")
                    .help("File to write the output to")
                    .takes_value(true),
            )
            .get_matches();
        let input_url = args
            .value_of("URL")
            .unwrap_or("gemini://192.168.0.106:1965/client_test.gmi");
        let input_url = Url::parse(input_url)?;
        assert_eq!(input_url.scheme(), "gemini");
        let socket_addr = input_url.socket_addrs(|| Some(1965))?;
        let output_file = args.value_of("output").map(|f| String::from(f));

        Ok(Config {
            input_url,
            socket_addr,
            output_file,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    pub status_code: u8,
    pub meta: String,
}

impl Header {
    pub fn get_status(&self) -> Option<Status> {
        use Status::*;
        match self.status_code {
            10..=19 => Some(Input),
            20..=29 => Some(Success),
            30..=39 => Some(Redirect),
            40..=49 => Some(TemporaryFailure),
            50..=59 => Some(PermanentFailure),
            60..=69 => Some(ClientCertificationRequired),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Input,
    Success,
    Redirect,
    TemporaryFailure,
    PermanentFailure,
    ClientCertificationRequired,
}

#[derive(Debug)]
pub struct MyErr {
    msg: &'static str,
}

#[derive(Debug)]
pub enum BodyData {
    Text(String),
    Binary(Vec<u8>),
}

impl From<&'static str> for MyErr {
    fn from(msg: &'static str) -> MyErr {
        MyErr { msg }
    }
}

pub fn get_response_header(response: &Vec<u8>) -> Result<Header, MyErr> {
    let mut header_iter = response.splitn(2, |c| *c == '\n' as u8);
    let header = String::from_utf8_lossy(header_iter.next().ok_or("ill-formed response")?);

    let mut header_fields_iter = header.splitn(2, ' ');
    let status_code = header_fields_iter
        .next()
        .ok_or("ill-formed header")?
        .parse::<u8>()
        .map_err(|_| "ill-formed status code")?;

    let meta = header_fields_iter
        .next()
        .ok_or("ill-formed header")?
        .trim()
        .to_string();

    Ok(Header { status_code, meta })
}

pub fn get_response_body(meta: &str, response: &Vec<u8>) -> Result<BodyData, FromUtf8Error> {
    let body = response
        .splitn(2, |c| *c == '\n' as u8)
        .nth(1)
        .unwrap()
        .to_vec();
    match meta.starts_with("text/") {
        true => {
            let text_body = String::from_utf8(body);
            match text_body {
                Ok(s) => Ok(BodyData::Text(s)),
                Err(e) => Err(e),
            }
        }
        false => Ok(BodyData::Binary(body)),
    }
}

pub fn get_tls_stream(sock: &[SocketAddr]) -> TlsStream<TcpStream> {
    let connector = TlsConnector::builder()
        .use_sni(false)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream = TcpStream::connect(&*sock).unwrap();
    connector.connect("", stream).unwrap()
}

pub fn send_request(stream: &mut TlsStream<TcpStream>, url: &Url) -> Vec<u8> {
    let mut request = url.to_string();
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    res
}

#[cfg(test)]
mod tests {
    use crate::Status::Input;
    use crate::{get_response_header, Header, MyErr};

    #[test]
    fn bad_response() {
        let response = "asdfasdffasdf".bytes().collect();
        let header = get_response_header(&response);
        assert_eq!(header.is_err(), true);
    }

    #[test]
    fn not_utf8_response() {
        let response = vec![33, 0xff, 34];
        // let response = response.bytes().collect();
        let header = get_response_header(&response);
        assert_eq!(header.is_err(), true);
    }

    #[test]
    fn valid_headers() {
        let response_list = vec![
            (crate::Status::Input, "10", "input prompt goes here"),
            (crate::Status::Success, "20", "text/gemini"),
            (crate::Status::Redirect, "30", "gemini://example.org/"),
            (
                crate::Status::TemporaryFailure,
                "40",
                "additional failure information",
            ),
            (
                crate::Status::PermanentFailure,
                "50",
                "additional failure information",
            ),
            (
                crate::Status::ClientCertificationRequired,
                "60",
                "additional certificate information",
            ),
        ];
        for line in response_list {
            let mut response = String::from(line.1);
            response.push(' ');
            response.push_str(line.2);
            response.push_str("\r\n");
            let response = response.bytes().collect();

            let header = get_response_header(&response);
            match &header {
                Ok(r) => {
                    assert_eq!(r.get_status().unwrap(), line.0);
                    assert_eq!(r.status_code, line.1.parse().unwrap());
                    assert_eq!(r.meta, line.2);
                }
                _ => panic!("bogus"),
            }
        }
        // let response = "10 input prompt goes here\r\n";
    }
}
