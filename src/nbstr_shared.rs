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

use Nbstr;
use nbstr::{MAX_LENGTH,MAX_STACK,LITERAL,BOX};
use utf8_char::Utf8Char;
extern crate std;
use std::cmp::Ordering;
use std::ops::Deref;
use std::str as Str;
use std::{mem,slice,ptr, fmt,hash};
use std::borrow::{Borrow,Cow};



/// Protected methods used by the impls below.
pub trait Protected {
    /// create new of this variant with possibly uninitialized data
    fn new(u8) -> Self;
    /// store this str, which is either &'static or boxed
    fn with_pointer(u8,  &str) -> Self;
    /// From<Utf8Char>
    fn from_1_utf8(Utf8Char) -> Self;
    /// From<char>
    fn from_1_utf32(char) -> Self;

    fn variant(&self) -> u8;
    /// get the area of self where (length,pointer)|inline is.
    fn data(&mut self) -> &mut [u8];
    /// the root of AsRef,Borrow and Deref.
    fn get_slice(&self) -> &[u8];
}



  ////////////////////
 // public methods //
////////////////////

impl Nbstr {
    #[cfg(feature="unstable")]
    /// Get the max length a Nbstr can store,
    ///  as some compile-time features might limit it.
    pub const MAX_LENGTH: usize = MAX_LENGTH;

    /// Get the max length a Nbstr can store,
    ///  as some compile-time features might limit it.
    ///
    /// This method will get deprecated once associated consts become stable.
    pub fn max_length() -> usize {
        MAX_LENGTH
    }

    // keeping all public methods under one impl gives cleaner rustdoc
    /// Create a Nbstr from a borrowed str with a limited lifetime.
    /// If the str is short enough it will be stored the inside struct itself and not boxed.
    pub fn from_str(s: &str) -> Self {
        Self::try_stack(s).unwrap_or_else(|| s.to_owned().into() )
    }
}


  ////////////////
 //Constructors//
////////////////

impl Default for Nbstr {
    fn default() -> Self {
        let s = unsafe{ slice::from_raw_parts(1 as *const u8, 0) };//pointer must be nonzero
        Self::with_pointer(LITERAL, unsafe{ mem::transmute(s) })
    }
}
impl From<&'static str> for Nbstr {
    fn from(s: &'static str) -> Self {
        Self::with_pointer(LITERAL, s)
    }
}
impl Nbstr {
    fn try_stack(s: &str) -> Option<Self> {match s.len() as u8 {
        // Cannot have stack str with length 0, as variant might be NonZero
        0 => Some(Self::default()),
        1...MAX_STACK => {
            let mut z = Self::new(s.len() as u8);
            for (d, s) in z.data().iter_mut().zip(s.bytes()) {
                *d = s;
            }
            Some(z)
        },
        _ => None,
    }}
}
impl From<Box<str>> for Nbstr {
    fn from(s: Box<str>) -> Self {
        // Don't try stack; users might turn it back into a box later
        let z = if s.is_empty() {Self::default()}// Make it clear we don't own any memory.
                else {Self::with_pointer(BOX, &s)};
        mem::forget(s);
        return z;
    }
}
impl From<String> for Nbstr {
    fn from(s: String) -> Self {
        if s.capacity() != s.len() {// into_boxed will reallocate
            if let Some(inline) = Self::try_stack(&s) {
                return inline;// and drop s
            }
        }
        return Self::from(s.into_boxed_str());
    }
}
impl From<Cow<'static, str>> for Nbstr {
    fn from(cow: Cow<'static, str>) -> Self {match cow {
        Cow::Owned(owned) => Self::from(owned),
        Cow::Borrowed(borrowed) => Self::from(borrowed),
    }}
}
impl From<char> for Nbstr {
    fn from(c: char) -> Self {
        Self::from_1_utf32(c)
    }
}

impl Clone for Nbstr {
    fn clone(&self) -> Self {
        if self.variant() == BOX {// try stack
            Nbstr::from_str(self.deref())
        } else {// copy the un-copyable
            unsafe{ ptr::read(self) }
        }
    }
    fn clone_from(&mut self,  from: &Self) {
        // keep existing box if possible
        if self.variant() == BOX  &&  self.len() == from.len() {
            unsafe{ ptr::copy_nonoverlapping( from.as_ptr(),
                                              mem::transmute(self.as_ptr()),
                                              self.len()
                                             )};
        } else {
            *self = from.clone();
        }
    }
}


  ///////////
 //Getters//
///////////

impl AsRef<[u8]> for Nbstr {
    fn as_ref(&self) -> &[u8] {
        self.get_slice()
    }
}
impl AsRef<str> for Nbstr {
    fn as_ref(&self) -> &str {
        let bytes: &[u8] = self.as_ref();
        unsafe{ Str::from_utf8_unchecked( bytes )}
    }
}
impl Deref for Nbstr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl Borrow<[u8]> for Nbstr {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}
impl Borrow<str> for Nbstr {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}


  /////////////////
 //Common traits//
/////////////////

impl hash::Hash for Nbstr {
    fn hash<H:hash::Hasher>(&self,  h: &mut H) {
        self.deref().hash(h);
    }
}
impl fmt::Display for Nbstr {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.deref(), fmtr)
    }
}
impl PartialOrd for Nbstr {
    fn partial_cmp(&self,  rhs: &Self) -> Option<Ordering> {
        self.deref().partial_cmp(rhs.deref())
    }
}
impl Ord for Nbstr {
    fn cmp(&self,  rhs: &Self) -> Ordering {
        self.deref().cmp(rhs.deref())
    }
}
impl PartialEq for Nbstr {
    fn eq(&self,  rhs: &Self) -> bool {
        self.deref() == rhs.deref()
    }
} impl Eq for Nbstr {}

/// Displays how the string is stored by prepending "stack: ", "literal: " or "boxed: ".
impl fmt::Debug for Nbstr {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}: {}", match self.variant() {
            1...MAX_STACK => "stack",
            LITERAL => "literal",
            BOX => "boxed",
            _ => unreachable!("Unknown variant of Nbstr: {}", self.variant())
        }, self.deref())
    }
}


  ///////////////
 //Destructors//
///////////////

/// Returns Some if z contains a Box
pub fn take_box(z: &mut Nbstr) -> Option<Box<str>> {
    if z.variant() == BOX {
        // I asked on #rust, and transmuting from & to mut is apparently undefined behaviour.
        // Is it really in this case?
        let s: *mut str = unsafe{ mem::transmute(z.get_slice()) };
        // Cannot just assign default; then rust tries to drop the previous value!
        //  .. which then calls this function.
        mem::forget(mem::replace(z,  Nbstr::default()));
        Some(unsafe{ Box::from_raw(s) })
    } else {
        None
    }
}

impl From<Nbstr> for Box<str> {
    fn from(mut z: Nbstr) -> Box<str> {
        take_box(&mut z).unwrap_or_else(|| z.deref().to_owned().into_boxed_str() )
    }
}
impl From<Nbstr> for String {
    fn from(mut z: Nbstr) -> String {
        take_box(&mut z)
            .map(|b| b.into_string() )
            .unwrap_or_else(|| z.deref().to_owned() )
    }
}
impl From<Nbstr> for Cow<'static, str> {
    fn from(mut z: Nbstr) -> Cow<'static, str> {
        take_box(&mut z)
            .map(|b| Cow::from(b.into_string()) )
            .unwrap_or_else(||
                if z.variant() == LITERAL {
                    let s: &'static str = unsafe{ mem::transmute(z.deref()) };
                    Cow::from(s)
                } else {
                    Cow::from(z.deref().to_owned())
                }
             )
    }
}
#[cfg(not(test))]// Bugs in drop might cause stack overflow in suprising places.
                //  The tests below should catch said bugs.
impl Drop for Nbstr {
    fn drop(&mut self) {
        let _ = take_box(self);
    }
}



  //////////////////////////////////////////////////////////////////////
 // Tests that need private access or tests code that use cfg!(test) //
//////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use nbstr::{Nbstr, MAX_STACK};
    use super::*;
    use std::ops::Deref;
    use std::str as Str;
    use std::{mem,slice};

    const STR: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    #[test]
    fn literal() {
        let mut z = Nbstr::from(STR);
        assert_eq!(z.deref().len(), STR.len());
        assert_eq!(z.deref().as_ptr(), STR.as_ptr());
        assert_eq!(z.deref(), STR);
        assert_eq!(take_box(&mut z), None);
    }
    #[test]
    fn stack() {
        let s = "abc";
        let mut z = Nbstr::from_str(s);
        assert_eq!(z.deref().len(), s.len());
        assert_eq!(z.deref().as_ptr() as usize, z.data().as_ptr() as usize);
        assert_eq!(z.deref(), s);
        assert_eq!(take_box(&mut z), None);
    }
    #[test]
    fn boxed() {
        let b: Box<str> = STR.to_string().into_boxed_str();
        let len = b.len();
        let ptr = b.as_ptr();
        let b2 = b.clone();
        let mut z = Nbstr::from(b);
        assert_eq!(z.deref().len(), len);
        assert_eq!(z.deref().as_ptr(), ptr);
        assert_eq!(z.deref(), STR);
        assert_eq!(take_box(&mut z), Some(b2.clone()));
        assert_eq!(take_box(&mut Nbstr::from_str(STR)), Some(b2.clone()));
    }

    #[test]
    fn nuls() {// Is here because MAX_STACK
        let zeros_bytes = [0; MAX_STACK as usize];
        let zeros_str = Str::from_utf8(&zeros_bytes).unwrap();
        let zeros = Nbstr::from_str(zeros_str);
        assert_eq!(zeros.deref(), zeros_str);
        assert!(Some(zeros).is_some());
    }
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]// from arithmetic overflow or explicit panic
    fn too_long() {// Is here because no_giants has a custom panic message for tests,
                  //   because the normal one would segfault on the invalid test str.
        let b: &[u8] = unsafe{slice::from_raw_parts(1 as *const u8,  1+Nbstr::max_length() )};
        let s: &'static str = unsafe{ mem::transmute(b) };
        mem::forget(Nbstr::from(s));
    }
}
