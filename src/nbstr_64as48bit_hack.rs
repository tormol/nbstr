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

use utf8_char::Utf8Char;
use nbstr_shared::Protected;
extern crate std;
use std::{mem,slice};
extern crate core;
use self::core::nonzero::NonZero;


pub const MAX_LENGTH: usize = 0x0000_ffff_ffff_ffff;
#[allow(dead_code)]
/// NonZero, never used
pub const NONE: u8 = 0;
/// 1...12 => stack string with length n
pub const MAX_STACK: u8 = 12;
/// &'static str: 48bit pointer, 48bit size
pub const LITERAL: u8 = MAX_STACK+1;
/// Box<str>: 48bit pointer, 48bit size
pub const BOX: u8 = MAX_STACK+2;
// empty strings are stored as LITERAL with non-NULL but possibly invalid pointer and zero length
//     (slices cannot have NULL pointers)


const MORE_THAN_48_BITS : &'static str =
"Your systems implementation of x86_64 can use more than 48 bits of its pointers;\
 Please inform the maintainers of the 'nbstr' crate.\
 In the meantime, you can stop using the 64as48bit_hack feature.";



/// A lean `Cow<'static, str>` that cannot be written to.
#[unsafe_no_drop_flag]// is set to empty literal during drop()
pub struct Nbstr {
    variant: NonZero<u8>,
    data: [u8; 12],
}


  //////////////////
 //Helper methods//
//////////////////

fn from_parts(variant: u8,  data: [u8; 12]) -> Nbstr {
    Nbstr{variant: unsafe{ NonZero::new(variant) },  data: data}
}
/// assumes non-stack
unsafe fn set_ptr(z: &mut Nbstr,  s: *const u8) {
    let ptr = s as usize;
    if cfg!(debug_assertions)  &&  ptr > 0x0000_7fff_ffff_ffff_usize
                               &&  ptr < 0xffff_8000_0000_0000_usize {
        panic!(MORE_THAN_48_BITS);
    }
    z.data[06] = (ptr<<00) as u8;
    z.data[07] = (ptr>>08) as u8;
    z.data[08] = (ptr>>16) as u8;
    z.data[09] = (ptr>>24) as u8;
    z.data[10] = (ptr>>32) as u8;
    z.data[11] = (ptr>>40) as u8;
}
fn get_ptr(z: &Nbstr) -> *const u8 {
    if *z.variant > MAX_STACK {
        let signed : *const isize = unsafe{ mem::transmute(z.data[4..].as_ptr())};
        let shifted = unsafe{*signed} >> 16;//sign extension
        shifted as *const u8
    } else {
        &z.data as *const u8
    }
}
/// assumes non-stack
unsafe fn set_len(z: &mut Nbstr,  len: usize) {
    if cfg!(debug_assertions)  &&  len > 0x0000_ffff_ffff_ffff_usize {
        panic!(MORE_THAN_48_BITS);
    }
    z.data[0] = (len>>00) as u8;
    z.data[1] = (len>>08) as u8;
    z.data[2] = (len>>16) as u8;
    z.data[3] = (len>>24) as u8;
    z.data[4] = (len>>32) as u8;
    z.data[5] = (len>>40) as u8;
}
fn get_len(z: &Nbstr) -> usize {
    if *z.variant > MAX_STACK {
        let location = z.data.as_ptr();
        let len : *const usize = unsafe{ mem::transmute(location) };
        unsafe{*len & 0x0000_ffff_ffff_ffff}
    } else {
        *z.variant as usize
    }
}


impl Protected for Nbstr {
    fn new(variant: u8) -> Self {
        from_parts(variant, unsafe{ mem::uninitialized() })
    }
    fn with_pointer(variant: u8,  s: &str) -> Self {
        let mut z = Self::new(variant);
        unsafe{ set_ptr(&mut z,  s.as_ptr()) };
        unsafe{ set_len(&mut z,  s.len()) };
        return z;
    }
    fn from_1_utf8(c: Utf8Char) -> Self {
        let mut z = Self::new(c.len() as u8);
        z.data = unsafe{ mem::transmute_copy(&c) };
        return z;
    }
    fn from_1_utf32(c: char) -> Self {
        let mut arr: [u8; MAX_STACK as usize] = unsafe{ mem::uninitialized() };
        let bytes = c.encode_utf8(&mut arr).unwrap();
        from_parts(bytes as u8, arr)
    }

    fn variant(&self) -> u8 {
        *self.variant
    }
    fn data(&mut self) -> &mut[u8] {
        &mut self.data
    }
    fn get_slice(&self) -> &[u8] {
        unsafe{ slice::from_raw_parts( get_ptr(self), get_len(self) )}
    }
}
