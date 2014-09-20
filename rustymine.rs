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
struct Packet {
    cmd: u8,
    value: PacketValue
}

enum PacketValue {
    Command,
    Data(Vec<u8>),
    String(String)
}

trait PacketStream {
    fn read_packet_command(&mut self) -> IoResult<Packet>;
    fn read_packet_data(&mut self) -> IoResult<Packet>;
    fn read_packet_string(&mut self) -> IoResult<Packet>;
    fn write_packet(&mut self, packet: Packet) -> IoResult<()>;
}

impl Packet {
    fn len(& self) -> uint {
        match self.value {
            Command => 1,
            Data(ref data) => 1 + data.len(),
            String(ref string) => {
                let len = string.len();
                1 + len + len.to_varint().len()
            }
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.push_all(self.len().to_varint().as_slice());
        response.push(self.cmd);
        match self.value {
            Command => {},
            Data(ref data) => {
                response.push_all(data.as_slice());
            },
            String(ref string) => {
                response.push_all(string.len().to_varint().as_slice());
                response.push_all(string.as_bytes());
            }
        }
        response
    }
}

impl PacketStream for TcpStream {
    fn read_packet_command(&mut self) -> IoResult<Packet> {
        let size = try!(self.read_varint());
        if size == 1 {
            let command = try!(self.read_byte());
            Ok(Packet {cmd: command, value: Command})
        } else {
            Err(IoError {
                kind: InvalidInput,
                desc: "Command packet with a size greater than one",
                detail: None
            })
        }
    }

    fn read_packet_data(&mut self) -> IoResult<Packet> {
        let size = try!(self.read_varint());
        let command = try!(self.read_byte());
        let data = try!(self.read_exact(size - 1));
        Ok(Packet {cmd: command, value: Data(data)})
    }

    fn read_packet_string(&mut self) -> IoResult<Packet> {
        try!(self.read_varint());
        let command = try!(self.read_byte());
        let string_size = try!(self.read_varint());
        let data = try!(self.read_exact(string_size));
        match std::str::from_utf8(data.as_slice()) {
            Some(string) => Ok(Packet {
                cmd: command,
                value: String(string.to_string())
            }),
            None => Err(IoError {
                kind: InvalidInput,
                desc: "Command packet with a size greater than one",
                detail: None
            })
        }
    }

    fn write_packet(&mut self, packet: Packet) -> IoResult<()> {
        self.write(packet.as_bytes().as_slice())
    }
}

fn process_stream(mut stream: TcpStream) -> IoResult<()> {
    let message = "{\"description\":\"Server is offline\",\"players\":{\"max\":0,\"online\":0},\"version\":{\"name\":\"1.8\",\"protocol\":47}}";

    let ip = try!(stream.peer_name()).ip;
    let packet = try!(stream.read_packet_data());
    match packet {
        Packet { cmd: 0, value: Data(data) } => {
            match data[13] {
                // Query
                1 => {
                    // We don't need anything from the second packet sent
                    try!(stream.read_packet_command());

                    // Send back the server status as JSON
                    try!(stream.write_packet(Packet {cmd: 0, value: String(message.to_string())}));

                    // Then receive and respond to the ping packet
                    let ping = try!(stream.read_packet_data());
                    try!(stream.write_packet(ping));
                },
                // Login
                2 => {
                    match stream.read_packet_string().unwrap() {
                        Packet {cmd: _, value: String(name)} => {
                            println!("Connection from {} ({})", name, ip);
                        },
                        _ => println!("Invalid login packet")
                    }

                },
                _ => println!("Invalid query packet")
            }
        },

        // Everything else
        _ => println!("Unknown packet")
    }
    Ok(())
}

fn main() {
    let mut acceptor = TcpListener::bind("0.0.0.0", 25565).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() process_stream(opt_stream.unwrap()).unwrap())
    }
}