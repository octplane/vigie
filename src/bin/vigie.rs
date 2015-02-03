#![feature(io)]
#![feature(core)]
#![feature(collections)]
#![feature(std_misc)]

extern crate linenoise;
extern crate url;

use std::time::Duration;
use std::old_io::TcpStream;
use url::Url;

use std::old_io::{IoResult};
use std::old_io::net::ip::{SocketAddr, Ipv4Addr};
use std::old_io::net::addrinfo::get_host_addresses;

type ProbeResult = Result<Duration, ProbeError>;

#[derive(Debug)]
enum ProbeError {
    EarlyError,
    ResolveError,
    ConnectError,
    ReadError,
    WriteError,
    PositiveMissing,
    NegativePresent,
}

struct ProbeOk;



fn url_to_socket_addr(host: &str, port: u16) -> IoResult<SocketAddr> {
    // Just grab the first IPv4 address
    let addrs = try!(get_host_addresses(host));
    let addr = addrs.into_iter().find(|&a| {
        match a {
            Ipv4Addr(..) => true,
            _ => false
        }
    });
    let addr = addr.unwrap();
    return Ok(SocketAddr{ip:addr, port: port});
}

fn http_path(url: &Url) -> String {
    let qr = match url.query {
        None => "".to_string(),
        Some(ref q) => format!("?{}", q),
    };

    let ret = format!("{}{}", url.serialize_path().unwrap(), qr);

    return ret.to_string();
}

fn get(url: &str, positive: Option<String>, negative: Option<String> ) -> Result<ProbeOk, ProbeError> {

    let u = Url::parse(url).unwrap();

    let port = match u.port() {
        None => 80,
        Some(i) => i,
    };

    let http_request = format!("GET {} HTTP/1.0\r\nHOST: {}\r\n\r\n", http_path(&u), u.domain().unwrap());

    match url_to_socket_addr(u.domain().unwrap(), port) {
      Ok(remote_addr) => {
        let c_timeout = Duration::seconds(2);
        match TcpStream::connect_timeout(remote_addr, c_timeout) {
          Ok(mut stream) => {
            stream.set_timeout(Some(5000));
            match stream.write_all(http_request.as_bytes()) {
              Ok(()) => {
                  let r = match stream.read_to_end() {
                    Ok(content_vec) => {
                      let content = String::from_utf8(content_vec).unwrap();
                      let mut res = Ok(ProbeOk);
                      if positive.is_some() && ! content.contains(&positive.unwrap()) {
                        res = Err(ProbeError::PositiveMissing);
                      }
                      if negative.is_some() && content.contains(&negative.unwrap()) {
                        res = Err(ProbeError::NegativePresent);
                      }
                      res
                    },
                    Err(_) => Err(ProbeError::ReadError)
                  };
                  stream.close_read().ok();
                  stream.close_write().ok();
                  return r;
                },
                Err(_) => Err(ProbeError::WriteError)

            }
          },
          Err(_) => return Err(ProbeError::ConnectError)
        }
      }
      Err(_) => return Err(ProbeError::ResolveError)
    }
}


fn http_probe(url: &str, positive: Option<String>, negative: Option<String> ) -> ProbeResult {


    let mut res: Result<ProbeOk, ProbeError> = Err(ProbeError::EarlyError);

    let d = Duration::span(|| {
      res = get(url, positive, negative);
    });

    return match res {
        Ok(ProbeOk) => Ok(d),
        Err(e) => Err(e),
    };

}

fn main() {
  match http_probe("http://q.golden-genie.eu/", Some("Golden Genie".to_string()), None) {
    Ok(duration) => println!("Ok, test took {}", duration),
    Err(e) => println!("Error during probe: {:?}", e),
  }
}
