use std::{
    io::{BufRead, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use bufstream::BufStream;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Args {
    /// bind the service to this tcp port, default 5555
    #[arg(short, long, default_value = "5555")]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let s = format!("0.0.0.0:{}", args.port)
        .parse::<SocketAddr>()
        .unwrap();
    println!("Listening to {s}");
    let listener = TcpListener::bind(s).unwrap();
    for incoming in listener.incoming() {
        match incoming {
            Ok(incoming) => {
                thread::spawn(|| prime_time(incoming));
            }

            Err(e) => eprintln!("error {e}"),
        }
    }
}

#[derive(Deserialize)]
struct Request {
    method: String,
    number: serde_json::Number,
}
#[derive(Serialize)]
struct Response {
    method: String,
    prime: bool,
}

fn prime_time(stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();

    println!("{peer_addr} - connected!");

    let mut stream = BufStream::new(stream);

    loop {
        let mut line = String::new();
        if let Ok(count) = stream.read_line(&mut line) {
            if count == 0 {
                // EOF: graceful shutdown
                break;
            }
            if let Ok(request) = serde_json::from_str::<Request>(&line) {
                if request.method == "isPrime" {
                    // read more requests

                    // note: unwrap will kill the thread and close the connection.
                    // which connection is useless if we cannot write to it ;)
                    stream
                        .write_all(
                            &serde_json::to_vec(&Response {
                                method: request.method,
                                prime: request
                                    .number
                                    .as_u64()
                                    .map(primes::is_prime)
                                    .unwrap_or(false),
                            })
                            .unwrap(),
                        )
                        .unwrap();
                    stream.write(&['\n' as u8]).unwrap();
                    stream.flush().unwrap();
                    continue;
                }
            }
        }
        // malformed request
        eprintln!("{peer_addr} - malformed!");
        let _ = stream.write_all(b"malformed\n");
        break;
    }
    eprintln!("{peer_addr} - closing connection.")
}
