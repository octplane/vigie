extern crate linenoise;
extern crate http;
extern crate url;
extern crate time;

use http::client::RequestWriter;
use http::method::Get;
use url::Url;

use std::time::Duration;


type ProbeResult = Result<Duration, ProbeError>;

#[deriving(Show)]
enum ProbeError {
    EarlyError,
    RequestError,
    ReadError,
    FullReadError
}

struct ProbeOk;

fn http_probe(url: &str) -> ProbeResult {


    let mut res: Result<ProbeOk, ProbeError> = Err(ProbeError::EarlyError);

    let d = Duration::span(|| {

        let url = Url::parse(url).unwrap();
        res = match RequestWriter::new(Get, url) {
            Ok(request) => match request.read_response() {
                    Ok(response) => {
                        let mut r = response;
                        match r.read_to_end() {
                            Ok(_) => Ok(ProbeOk),
                            Err(_) => Err(ProbeError::FullReadError),
                        }
                    },
                    Err((_, _)) => Err(ProbeError::ReadError)
                },
            Err(_) => Err(ProbeError::RequestError),
        };
    });

    return match res {
        Ok(ProbeOk) => Ok(d),
        Err(e) => Err(e),
    };

}

fn main() {



    loop {
	    let val = linenoise::input("vigie> ");
        match val {
            None => { break }
            Some(input) => {
                println!("Probe {}", input);
                match http_probe(input.as_slice()) {
                    Ok(duration) => println!("Took {}ms.", duration.num_milliseconds()),
                    Err(err) => println!("Failed {}", err)
                }
                linenoise::history_add(input.as_slice());
                if input.as_slice() == "clear" {
                	linenoise::clear_screen();
                }
            }
        }
    }
}