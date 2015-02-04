#![feature(io)]
#![feature(os)]
#![feature(core)]
#![feature(collections)]
#![feature(std_misc)]

extern crate linenoise;
extern crate url;
extern crate getopts;

use std::time::Duration;
use std::old_io::TcpStream;
use url::Url;

use std::old_io::{IoResult, IoError};
use std::old_io::net::ip::{SocketAddr, Ipv4Addr};
use std::old_io::net::addrinfo::get_host_addresses;
use std::error::FromError;

use getopts::Options;
use std::os;

//use std::thread::Thread;


type ProbeResult = Result<Duration, ProbeError>;

#[derive(Debug)]
enum ProbeError {
  IoError(IoError),
  ResolveError,
  EarlyError,
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

impl FromError<IoError> for ProbeError {
    fn from_error(err: IoError) -> ProbeError {
        ProbeError::IoError(err)
    }
}

fn first(needle: &String , haystack: &Vec<u8>) -> Option<usize> {
  let ne = needle.as_bytes();
  for x in 0..haystack.len() {
    if haystack[x..].starts_with(ne) {
      return Some(x);
    }
  }
  None
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
        let mut stream = try!(TcpStream::connect_timeout(remote_addr, c_timeout));
        stream.set_timeout(Some(5000));
        try!(stream.write_all(http_request.as_bytes()));
        let content_vec =  try!(stream.read_to_end());
        let mut res = match negative {
          Some(n) => match first(&n, &content_vec) {
            Some(_) => Err(ProbeError::NegativePresent),
            None => Ok(ProbeOk)
          },
          None => Ok(ProbeOk)
        };

        res = match res {
          Err(e) => Err(e),
          Ok(ProbeOk) => match positive {
            Some(p) => match first(&p, &content_vec) {
              Some(_) => Ok(ProbeOk),
              None => Err(ProbeError::PositiveMissing)
            },
            None => Ok(ProbeOk)
          },
        };

        stream.close_read().ok();
        stream.close_write().ok();

        res
      },
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

fn callback(input: &str) -> Vec<String> {
  let mut ret : Vec<&str>;
  if input.starts_with("m") {
    ret = vec!["monitor", "suggestion2", "suggestion-three"];
  } else {
    ret = vec!["wot"];
  }
  return ret.iter().map(|s| s.to_string()).collect();
}


fn run_shell() {
  linenoise::set_callback(callback);
  loop {
    let val = linenoise::input("vigie > ");
    match val {
      None => { break }
      Some(input) => {
        println!("> {}", input);
        linenoise::history_add(input.as_slice());
        if input.as_slice() == "clear" {
          linenoise::clear_screen();
        }
      }
    }
  }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(brief.as_slice()));
}

// fn echo(mut req: Request, mut res: Response) {
//   match req.uri {
//     AbsolutePath(ref path) => match (&req.method, &path[]) {
//       (&Get, "/") | (&Get, "/echo") => {
//         let out = b"Try POST /echo";

//         res.headers_mut().set(ContentLength(out.len() as u64));
//         let mut res = try_return!(res.start());
//         try_return!(res.write_all(out));
//         try_return!(res.end());
//         return;
//       },
//     (&Post, "/echo") => (), // fall through, fighting mutable borrows
//     _ => {
//         *res.status_mut() = hyper::NotFound;
//         try_return!(res.start().and_then(|res| res.end()));
//         return;
//       }
//     },
//     _ => {
//       try_return!(res.start().and_then(|res| res.end()));
//       return;
//     }
//   };

//   let mut res = try_return!(res.start());
//   try_return!(copy(&mut req, &mut res));
//   try_return!(res.end());
// }



fn main() {
  let args: Vec<String> = os::args();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optflag("s", "shell", "start shell");
  opts.optflag("h", "help", "print this help menu");
  let matches = match opts.parse(args.tail()) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
  };
  if matches.opt_present("h") {
      print_usage(program.as_slice(), opts);
      return;
  }

  if matches.opt_present("s") {
      run_shell();
      return;
  }

  // let http = Thread::spawn({ move ||
  //   let server = Server::http(Ipv4Addr(127, 0, 0, 1), 1337);
  //   let mut listening = server.listen(echo).unwrap();

  // });


  match http_probe("http://q.golden-genie.eu/", Some("Golden Genie".to_string()), None) {
    Ok(duration) => println!("Ok, test took {}", duration),
    Err(e) => println!("Error during probe: {:?}", e),
  }
}
