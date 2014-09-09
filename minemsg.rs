use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

// 0x6d - "A Minecraft Server"
// 0x6a - "nullreff server"
// 0x64 - "test test"
// 0x5c - "a"

fn main() {
    let message = "{\"description\":\"hello world\",\"players\":{\"max\":20,\"online\":0},\"version\":{\"name\":\"1.8\",\"protocol\":47}}";
    let length = message.len().to_u16().unwrap();

    let mut acceptor = TcpListener::bind("0.0.0.0", 25565).listen().unwrap();
    println!("listening started, ready to accept");
    println!("Message length: {}", length);
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let cmd = stream.read_byte().unwrap();
            match cmd {
                0x0F => {
                    let request = stream.read_exact(17).unwrap();
                    println!("Query {}", request);
                    let mut response = Vec::new();
                    response.push(length.to_u8().unwrap() + 2);
                    response.push((length >> 8).to_u8().unwrap());
                    response.push((length & 0xFF).to_u8().unwrap());
                    response.push_all(message.as_bytes());
                    stream.write(response.as_slice()).unwrap();

                    let ping = stream.read_exact(10).unwrap();
                    stream.write(ping.as_slice()).unwrap();
                },

                // Everything else
                _ => { println!("Unknown packet {}", cmd); }
            }
        })
    }
}
