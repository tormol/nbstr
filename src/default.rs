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
use std::mem;

#[cfg(feature="unstable")]
extern crate core;
#[cfg(not(feature="unstable"))]
mod core {
    pub mod nonzero {
        use std::ops::Deref;

        /// A stable minimal stand-in for NonZero.
        #[derive(Clone)]
        pub struct NonZero<T> {
            zeroable: T
        }
        impl<T> NonZero<T> {
            pub unsafe fn new(not_zero: T) -> Self {
                NonZero{zeroable: not_zero}
            }
        }
        impl<T> Deref for NonZero<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &self.zeroable
            }
        }
    }
}
use self::core::nonzero::NonZero;



#[cfg(target_pointer_width="16")]
const POINTER_BYTES: usize = 2;
#[cfg(target_pointer_width="32")]
const POINTER_BYTES: usize = 4;
#[cfg(target_pointer_width="64")]
const POINTER_BYTES: usize = 8;

const DATA_SIZE: usize = 2*POINTER_BYTES;
pub const MAX_LENGTH: usize = 0xffff_ffff_ffff_ffff_u64 as usize;

#[allow(dead_code)]
/// NonZero, never used
pub const NONE: u8 = 0;
/// 1...MAX_STACK => stack string with length n
pub const MAX_STACK: u8 = DATA_SIZE as u8;
/// &'static str
pub const LITERAL: u8 = MAX_STACK+1;
/// Box<str>
pub const BOX: u8 = MAX_STACK+2;
// empty strings are stored as LITERAL with non-NULL but possibly invalid pointer and zero length
//     (slices cannot have NULL pointers)



/// A lean `Cow<'static, str>` that cannot be written to.
#[cfg_attr(feature="unstable", unsafe_no_drop_flag)]//is set to empty literal during drop()
pub struct Nbstr {
    variant: NonZero<u8>,
    data: [u8; DATA_SIZE],
}


fn from_parts(variant: u8,  data: [u8; DATA_SIZE]) -> Nbstr {
    Nbstr{variant: unsafe{ NonZero::new(variant) },  data: data}
}


impl Protected for Nbstr {
    fn new(variant: u8) -> Self {
        from_parts(variant, unsafe{ mem::uninitialized() })
    }
    fn with_pointer(variant: u8,  s: &str) -> Self {
        from_parts(variant, unsafe{mem::transmute(s)} )
    }

    fn variant(&self) -> u8 {
        *self.variant
    }
    fn data(&mut self) -> &mut[u8] {
        &mut self.data
    }
    fn get_slice(&self) -> &[u8] {
        if *self.variant > MAX_STACK {
            unsafe{ mem::transmute(self.data) }
        } else {
            &self.data[..*self.variant as usize]
        }
    }
    unsafe fn get_mut_slice(&mut self) -> *mut[u8] {
        mem::transmute(self.data)
    }
}
