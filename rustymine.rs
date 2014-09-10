/* Rustymine - Barebones Minecraft server in Rust
 *
 * Copyright (c) 2014 Ryan Mendivil <ryan@nullreff.net>
 * All rights reserved.
 * 
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 
 *   * Redistributions of source code must retain the above copyright notice,
 *     this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *   * Neither the name of Rustymine nor the names of its contributors may be
 *     used to endorse or promote products derived from this software without
 *     specific prior written permission.
 * 
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

static HOST:&'static str = "0.0.0.0";
static PORT:u16          = 25565;

fn main() {
    let message = "{\"description\":\"Server is offline\",\"players\":{\"max\":0,\"online\":0},\"version\":{\"name\":\"1.8\",\"protocol\":47}}";
    let length = message.len().to_u16().unwrap();

    let mut acceptor = TcpListener::bind(HOST, PORT).listen().unwrap();
    println!("Listening on {}:{}", HOST, PORT);
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
