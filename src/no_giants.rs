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

use shared::Protected;
extern crate std;
use std::{mem,slice};
extern crate core;
use self::core::nonzero::NonZero;



// const POINTER_BYTES: usize = mem::size_of::<usize>();
#[cfg(target_pointer_width="16")]
const POINTER_BYTES: usize = 2;
#[cfg(target_pointer_width="32")]
const POINTER_BYTES: usize = 4;
#[cfg(target_pointer_width="64")]
const POINTER_BYTES: usize = 8;

const SIZE: usize = 2*POINTER_BYTES;
// need to store DATA_SIZE+3 combinations
// even if you can subtract one, you need one extra bit.
// 16bit=4+2=3bit=shift 13, 32bit=8+2=4bit=shift 28, 64bit=16+2=5bit=shift 59
// start with 16 bits: 16.trailing_zeros()==4, which is one too much.
// when only one bit is set, n.leading_zeros()==n::BITS-1-n.trailing_zeros()
// const SHIFT_BITS: usize = (8*POINTER_BYTES).leading_zeros() as usize;
#[cfg(target_pointer_width="16")]
const SHIFT_BITS: usize = 13;
#[cfg(target_pointer_width="32")]
const SHIFT_BITS: usize = 28;
#[cfg(target_pointer_width="64")]
const SHIFT_BITS: usize = 59;

pub const MAX_LENGTH: usize = (1 << SHIFT_BITS) -1;

#[allow(dead_code)]
/// NonZero, never used
pub const NONE: u8 = 0;
/// 1...MAX_STACK => stack string with length n
pub const MAX_STACK: u8 = (SIZE-1) as u8;//one byte is used for variant
/// &'static str
pub const LITERAL: u8 = MAX_STACK+1;
/// Box<str>
pub const BOX: u8 = MAX_STACK+2;
// empty strings are stored as LITERAL with non-NULL but possibly invalid pointer and zero length
//     (slices cannot have NULL pointers)



/// A lean `Cow<'static, str>` that cannot be written to.
#[unsafe_no_drop_flag]// is set to empty literal during drop()
#[repr(C)]// endian-dependent order
pub struct Nbstr {
    // The byte that contains variant cannot be in the middle of an array.
    #[cfg(target_endian="big")]
    length: NonZero<usize>,
    pointer: *const u8,
    #[cfg(target_endian="little")]
    length: NonZero<usize>,
}


impl Protected for Nbstr {
    fn new(variant: u8) -> Self {
        Nbstr {
            length:  unsafe{ NonZero::new( (variant as usize) << SHIFT_BITS )},
            pointer:  unsafe{ mem::uninitialized() },
        }
    }
    fn with_pointer(variant: u8,  s: &str) -> Self {
        if cfg!(debug_assertions)  &&  s.len() > MAX_LENGTH {
            if cfg!(test) {// dereferencing the test string would segfault
                panic!(".len()={:x}, MAX_LENGTH={:x}", s.len(), MAX_LENGTH);
            } else {
                panic!("The string \"{}\"...\"{}\", with length {}, is too long for Nbstr.\n\
                        Disable the \"tag_len\" feature.", &s[..20], &s[s.len()-20..], s.len());
            }
        }
        let len = ((variant as usize) << SHIFT_BITS)  |  s.len();
        Nbstr {
            pointer: s.as_ptr(),
            length: unsafe{ NonZero::new(len) },
        }
    }

    fn variant(&self) -> u8 {
        (*self.length >> SHIFT_BITS) as u8
    }
    fn data(&mut self) -> &mut[u8] {
        let arr: &mut[u8; SIZE] = unsafe{ mem::transmute(self) };
        if cfg!(target_endian="big") {
            &mut arr[1..]
        } else {
            &mut arr[..SIZE-1]
        }
    }
    fn get_slice(&self) -> &[u8] {
        if self.variant() > MAX_STACK {
            unsafe{ slice::from_raw_parts(self.pointer,  *self.length & MAX_LENGTH ) }
        } else {
            let arr: &[u8; SIZE] = unsafe{ mem::transmute( self )};
            if cfg!(target_endian="little") {
                &arr[..self.variant() as usize]
            } else {
                &arr[1..1+self.variant() as usize]
            }
        }
    }
}
