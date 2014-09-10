/* Varint - Variable length integers implemented in Rust
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

use std::io::{IoResult, IoError};
use std::io::net::tcp::{TcpStream};

// https://developers.google.com/protocol-buffers/docs/encoding#varints
pub trait ReadVarint<T> {
    fn read_varint(&mut self) -> Result<uint, T>;
}

pub trait ToVarint {
    fn to_varint(&self) -> &[u8];
}

impl ToVarint for uint {
    // Takes 7 bits at a time from a number and splits them into a list of
    // bytes. The most significant bit is set on each byte that has another
    // coming after it.
    fn to_varint(&self) -> &[u8] {
        let mut remaining = self.clone();
        let mut result = Vec::new();
        loop {
            let part = (remaining & 0x7F).to_u8().unwrap(); // First 7 bits
            remaining >>= 7;
            if remaining == 0 {
                result.push(part | 0x80);
            } else {
                result.push(part);
                break;
            }
        }
        result.as_slice()
    }
}

impl ReadVarint<IoError> for TcpStream {
    // This takes one byte at a time from a reader as long as the most significant
    // bit is 1.  Combines the remaining 7 bits together to make a number.
    fn read_varint(&mut self) -> IoResult<uint> {
        let mut result:uint = 0;

        // uint's can only take up to 4 bytes
        for i in range(0, 4) {
            match self.read_byte() {
                Ok(part) => {
                    result &= (part & 0x7F).to_uint().unwrap() << i;
                    if (part >> 7) == 0 {
                        break;
                    }
                },
                Err(e) => return Err(e)
            }
        }
        Ok(result)
    }
}
