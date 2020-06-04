#[derive(Debug)]
pub struct Header {
    pub status_code: u8,
    pub meta: String,
}

impl Header {
    fn get_status(&self) -> Option<Status> {
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
enum Status {
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

impl From<&'static str> for MyErr {
    fn from(msg: &'static str) -> MyErr {
        MyErr { msg }
    }
}

pub fn get_response_header(response: &str) -> Result<Header, MyErr> {
    let response_lines = response.splitn(2, '\n').collect::<Vec<&str>>();
    let header = response_lines.get(0).ok_or("ill-formed response")?;

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

#[cfg(test)]
mod tests {
    use crate::{get_response_header, MyErr, Header};
    use crate::Status::Input;

    #[test]
    fn bad_response() {
        let response = "asdfasdffasdf";
        let header = get_response_header(response);
        assert_eq!(header.is_err(), true);
    }

    #[test]
    fn valid_input() {
        let response_list = vec![
            (crate::Status::Input, "10", "input prompt goes here"),
            (crate::Status::Success, "20", "text/gemini"),
            (crate::Status::Redirect, "30", "gemini://example.org/"),
            (crate::Status::TemporaryFailure, "40", "additional failure information"),
            (crate::Status::PermanentFailure, "50", "additional failure information"),
            (crate::Status::ClientCertificationRequired, "60", "additional certificate information"),
        ];
        for line in response_list {
            let mut response = String::from(line.1);
            response.push(' ');
            response.push_str(line.2);
            response.push_str("\r\n");

            let header = get_response_header(&response);
            match &header {
                Ok(r) => {
                    assert_eq!(r.get_status().unwrap(), line.0);
                    assert_eq!(r.status_code, line.1.parse().unwrap());
                    assert_eq!(r.meta, line.2);
                },
                _ => panic!("bogus"),
            }
        }
        // let response = "10 input prompt goes here\r\n";
    }
}
