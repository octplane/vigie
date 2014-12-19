extern crate linenoise;
extern crate url;

use std::time::Duration;
use std::io::TcpStream;
use url::Url;

use std::io::{IoError, IoResult};
use std::io::net::ip::{SocketAddr, Ipv4Addr, IpAddr};
use std::io::net::addrinfo::get_host_addresses;

type ProbeResult = Result<Duration, ProbeError>;

#[deriving(Show)]
enum ProbeError {
    EarlyError,
    RequestError,
    ReadError,
    FullReadError
}

struct ProbeOk;



fn url_to_socket_addr(host: &str) -> IoResult<SocketAddr> {
    // Just grab the first IPv4 address
    let addrs = try!(get_host_addresses(host));
    let addr = addrs.into_iter().find(|&a| {
        match a {
            Ipv4Addr(..) => true,
            _ => false
        }
    });
    let addr = addr.unwrap();
    return Ok(SocketAddr{ip:addr, port: 80});
}

fn http_path(url: &Url) -> String {
    let qr = match url.query {
        None => "".to_string(),
        Some(ref q) => format!("?{}", q),
    };

    let ret = format!("{}{}", url.serialize_path().unwrap(), qr);

    return ret.to_string();
}

fn http_probe(url: &str) -> IoResult<ProbeResult> {

    let u = Url::parse(url).unwrap();

    let port = match u.port() {
        None => 80,
        Some(i) => i,
    };

    let d = Duration::seconds(2);
    let http_host = format!("{}:{}", u.domain(), port);
    let http_request = format!("GET {} HTTP/1.0\nHOST: {}\n\n", http_path(&u), u.domain().unwrap());


    let remote_addr = try!(url_to_socket_addr(u.domain().unwrap()));

    let mut stream = try!(TcpStream::connect_timeout(remote_addr, d));
    println!("Connected");
    stream.set_timeout(Some(2000));
    stream.write(http_request.as_bytes());
    println!("Wrote");
    let re = stream.read_to_end();
    println!("{}", re)

    stream.close_read();
    stream.close_write();

    return Ok(Err(ProbeError::EarlyError));
}


// fn http_probe(url: &str) -> ProbeResult {


//     let mut res: Result<ProbeOk, ProbeError> = Err(ProbeError::EarlyError);

//     let d = Duration::span(|| {

//         let url = Url::parse(url).unwrap();
//         res = match RequestWriter::new(Get, url) {
//             Ok(request) => match request.read_response() {
//                     Ok(response) => {
//                         let mut r = response;
//                         match r.read_to_end() {
//                             Ok(_) => Ok(ProbeOk),
//                             Err(_) => Err(ProbeError::FullReadError),
//                         }
//                     },
//                     Err((_, _)) => Err(ProbeError::ReadError)
//                 },
//             Err(_) => Err(ProbeError::RequestError),
//         };
//     });

//     return match res {
//         Ok(ProbeOk) => Ok(d),
//         Err(e) => Err(e),
//     };

// }

fn main() {


    let _ = http_probe("http://www.zoy.org/path/?pipo");

    // loop {
	   //  let val = linenoise::input("vigie> ");
    //     match val {
    //         None => { break }
    //         Some(input) => {
    //             println!("Probe {}", input);
    //             match http_probe(input.as_slice()) {
    //                 Ok(Ok(duration)) => println!("Took {}ms.", duration.num_milliseconds()),
    //                 Ok(Err(err)) => println!("Failed {}", err),
    //                 Err(err) => println!("Failed {}", err),
    //             }
    //             linenoise::history_add(input.as_slice());
    //             if input.as_slice() == "clear" {
    //             	linenoise::clear_screen();
    //             }
    //         }
    //     }
    // }
}