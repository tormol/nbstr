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


//! A lean `Cow<'static, str>` that cannot be written to.
//!
//! Size is reduced by replacing `String` with `Box<str>``, which removes one `usize`, at the cost of some reallocation.
//!
//! To avoid boxing when possible, short `str`s can be stored inside the struct itself, replacing pointer and length.
//! The length of the short str is then stored as a part of the tag/discriminant, which is why Nbstr is a struct and not an enum.
//! The definition of 'short' depends on architecture and features.
//!
//! There are four variants of nbstr; See README for details.
//!
//! # Examples
//!
//! ```rust
//! extern crate nbstr;
//! use nbstr::Nbstr;
//!
//! #[derive(Default)]
//! struct Container {// <- no lifetime
//!     list: Vec<Nbstr>
//! }
//! impl Container {
//!     fn append<S:Into<Nbstr>>(&mut self,  s: S) {
//!         self.list.push(s.into());
//!     }
//! }
//! fn main() {
//!     let mut c = Container::default();
//!     c.append("foo");// &'static str
//!     {   // &str wouldn't work here since the strings goes out of scope before the Vec
//!         c.append(Nbstr::from_str(&("bar".to_string())));// is short enough to avoid allocating,
//!         c.append("baz".to_string());
//!     }
//!     println!("{:?}", c.list);
//! }
//! ```

// Overview:
// shared.rs: the public interface and code used in all variants.
// other: variant-specific code and implementation details.


// unstable features
#![cfg_attr(feature="unstable", feature(associated_consts,  nonzero, unsafe_no_drop_flag))]

#![warn(missing_docs)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

mod shared;

#[cfg(not(any(feature="no_giants", all(feature="64as48bit_hack", target_arch="x86_64"))))]
mod default;
#[cfg(all(feature="no_giants", not(all(feature="64as48bit_hack", target_arch="x86_64"))))]
mod no_giants;
#[cfg(all(feature="64as48bit_hack", target_arch="x86_64"))]
mod x64as48bit_hack;// cannot start with a number
mod nbstr {// rename variants
    #[cfg(not(any(feature="no_giants", all(feature="64as48bit_hack", target_arch="x86_64"))))]
    pub use default::*;
    #[cfg(all(feature="no_giants", not(all(feature="64as48bit_hackt", target_arch="x86_64"))))]
    pub use no_giants::*;
    #[cfg(all(feature="64as48bit_hack", target_arch="x86_64"))]
    pub use x64as48bit_hack::*;
}
pub use nbstr::Nbstr;
