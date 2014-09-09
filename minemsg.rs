use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

fn main() {
    let message = "{\"description\":\"Server is offline\",\"players\":{\"max\":0,\"online\":0},\"version\":{\"name\":\"1.8\",\"protocol\":47}}";
    let length = message.len().to_u16().unwrap();

    let mut acceptor = TcpListener::bind("0.0.0.0", 25565).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let ip = stream.peer_name().unwrap().ip;
            let cmd = stream.read_byte().unwrap();
            match cmd {
                0x0F => {
                    let request = stream.read_exact(15).unwrap();
                    match request[14] {
                        // Query
                        1 => {
                            stream.read_exact(2).unwrap();
                            let mut response = Vec::new();
                            response.push(length.to_u8().unwrap() + 2);
                            response.push((length >> 8).to_u8().unwrap());
                            response.push((length & 0xFF).to_u8().unwrap());
                            response.push_all(message.as_bytes());
                            stream.write(response.as_slice()).unwrap();

                            let ping = stream.read_exact(10).unwrap();
                            stream.write(ping.as_slice()).unwrap();
                        },
                        // Login
                        2 => {
                            stream.read_exact(2).unwrap();
                            let size = stream.read_byte().unwrap().to_uint().unwrap();
                            let name_vector = stream.read_exact(size).unwrap();
                            let name = std::str::from_utf8(name_vector.as_slice()).unwrap();
                            println!("Connection from {} ({})", name, ip);
                        },
                        _ => {}
                    }
                },

                // Everything else
                _ => { println!("Unknown packet {}", cmd); }
            }
        })
    }
}
