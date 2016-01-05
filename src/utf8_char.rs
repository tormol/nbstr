/* Copyright 2016 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

extern crate std;
use std::mem;
use std::ops::{Not, Deref};



#[derive(Clone,Copy)]
/// Store a `char` as UTF-8 so it can be borrowed as a `str`
///
/// Doesn't allocate, and has the same size as a char.
/// Cannot represent all 2^32-1 possible values, but can do all valid ones.
///
/// A helper for Nbstr that might be useful on its own while char.encode_utf8() is unstable.
pub struct Utf8Char {
    bytes: [u8; 4],
}


impl From<char> for Utf8Char {
    #[cfg(feature="unstable")]
    fn from(c: char) -> Self {
        let mut bytes : [u8; 4] = unsafe{ mem::uninitialized() };
        let _ = c.encode_utf8(&mut bytes);
        Utf8Char{bytes: bytes}
    }
    #[cfg(not(feature="unstable"))]//do it ourself
    fn from(c: char) -> Self {
        //.encode_utf8() relies on ::from_u32() to ensure valid values, so we do too
        let length = match c as u32 {
            // ASCII, the common case
            0...0x7f => return Utf8Char{ bytes: [c as u8, 0, 0, 0] },// 7 bits
            0x80...0x07_ff => 2,// 5+6 = 11 bits
            0x08_00...0xff_ff => 3,// 4+6+6 = 16 bits
            0x1_00_00...0x1f_ff_ff => 4,// 3+6+6+6 = 21 bits
            _ => 4// too big, .encode_utf8() accepts it, so we must too.
        };

        let c = (c as u32) << 6*(4-length);// left-align bytes
        let parts = [// convert to 6-bit bytes
                (c>>3*6) as u8,
                (c>>2*6) as u8,
                (c>>1*6) as u8,
                (c>>0*6) as u8,
            ];
        let mut all: u32 = unsafe{ mem::transmute(parts) };// make all bytes start with 10xx_xxxx
        all &= 0xbf_bf_bf_bf;// clear the next most significant bit
        all |= 0x80_80_80_80;// set the most significant bit
        // some of the bits might be unnessecarry, but it's not any slower, and is unaffected by endianness.
        let mut bytes: [u8; 4] = unsafe{ mem::transmute(all) };

        bytes[0] |= 0xff << 8-length;// store length
        bytes[0] &= Not::not(1u8 << 7-length);// clear the next bit after it
        Utf8Char{ bytes: bytes }
    }
}


impl Deref for Utf8Char {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe{ std::str::from_utf8_unchecked(&self.bytes[..self.len()]) }
    }
}


impl Utf8Char {
    /// Result is 1...4 and identical to .as_str().len()
    /// There is no .is_emty() because it would always return false, and be unused
    pub fn len(self) -> usize {
        match self.bytes[0].not().leading_zeros() {
            0 => 1,
            1 => panic!("Utf8Char has invalid utf-8"),
            n => n as usize,
        }
    }
}
