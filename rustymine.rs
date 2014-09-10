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

use std::io::{Listener, Acceptor, IoResult, IoError, InvalidInput};
use std::io::net::tcp::{TcpListener, TcpStream};
use varint::{ReadVarint, ToVarint};
mod varint;

// Most of this was put together using http://wiki.vg/Protocoland Wireshark captures.
enum Packet {
    CommandPacket(u8),
    DataPacket(u8, Vec<u8>),
    JsonPacket(u8, str)
}

impl Packet {
    fn len(&self) -> uint {
        match *self {
            CommandPacket(cmd) => 1,
            DataPacket(cmd, data) => 1 + data.len(),
            JsonPacket(cmd, json) => {
                let len = json.len();
                1 + len + len.to_varint().len()
            }
        }
    }

    fn as_bytes(self) -> Vec<u8> {
        let mut response = Vec::new();
        response.push_all(self.len().to_varint());
        match self {
            CommandPacket(cmd) => {
                response.push(cmd);
            },
            DataPacket(cmd, data) => {
                response.push(cmd);
                response.push_all(data.as_slice());
            },
            JsonPacket(cmd, json) => {
                response.push(cmd);
                response.push_all(json.len().to_varint());
                response.push_all(json.as_bytes());
            }
        }
        response
    }

    fn write_to(&self, stream: &mut TcpStream) -> IoResult<()> {
        stream.write(self.as_bytes().as_slice())
    }

    fn read_command(stream: &mut TcpStream) -> IoResult<Packet> {
        stream.read_varint().and_then(|size| {
            if size != 1 {
                Err(IoError {
                    kind: InvalidInput,
                    desc: "Command packet with a size greater than one",
                    detail: None
                })
            } else {
                stream.read_byte().map(|command| {
                    CommandPacket(command)
                })
            }
        })
    }

    fn read_data(stream: &mut TcpStream) -> IoResult<Packet> {
        stream.read_varint().and_then(|size| {
            stream.read_byte().and_then(|command| {
                stream.read_exact(size - 1).map(|data| {
                    DataPacket(command, data)
                })
            })
        })
    }

    fn read_json(stream: &mut TcpStream) -> IoResult<Packet> {
        stream.read_varint().and_then(|size| {
            stream.read_byte().and_then(|command| {
                stream.read_exact(size - 1).and_then(|data| {
                    match std::str::from_utf8(data.as_slice()) {
                        Some(&json) => Ok(JsonPacket(command, json)),
                        None => Err(IoError {
                            kind: InvalidInput,
                            desc: "Command packet with a size greater than one",
                            detail: None
                        })
                    }
                })
            })
        })
    }
}

fn main() {
    let message = "{\"description\":\"Server is offline\",\"players\":{\"max\":0,\"online\":0},\"version\":{\"name\":\"1.8\",\"protocol\":47}}";

    let mut acceptor = TcpListener::bind("0.0.0.0", 25565).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let ip = stream.peer_name().unwrap().ip;
            let DataPacket(cmd, data) = Packet::read_data(&mut stream).unwrap();
            match cmd {
                0x00 => {
                    match data[13] {
                        // Query
                        1 => {
                            // We don't need anything from the second packet sent
                            Packet::read_command(&mut stream).unwrap();

                            JsonPacket(0, *message).write_to(&mut stream).unwrap();

                            let ping = Packet::read_data(&mut stream).unwrap();
                            ping.write_to(&mut stream).unwrap();
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
