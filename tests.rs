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

use std::ops::Deref;
use std::borrow::{Borrow,Cow};
use std::hash::{Hash,SipHasher};
use std::fmt::{Display,Debug};
extern crate nbstr;
use nbstr::Nbstr;

/// Catches missing trait impls.
/// Unfortunately there is no way to prevent aditional public methods or traits.

pub trait NbstrTrait: Sized + Clone + Hash + Eq + Ord + Display + Debug//Debug prepends the str with the way it's stored
 +Default + From<&'static str>+From<Box<str>>+From<String>+From<Cow<'static,str>> + From<char>
 +Deref<Target=str> + Borrow<str>+Borrow<[u8]> + AsRef<str>+AsRef<[u8]>
 +Into<Box<str>>+Into<String>+Into<Cow<'static,str>>//actually, implement From<Nbstr> for Box<str> and String
{}
impl NbstrTrait for Nbstr {}

const A_FEW: &'static str = "\0eéaå𝛼 ∆θ≈π";
#[test]
fn from_char() {
    for c in A_FEW.chars() {
        let mut s = String::new();
        s.push(c);
        assert_eq!(Nbstr::from(c).deref(), s);
    }
}
#[test]
fn from_a_few() {
    assert_eq!(Nbstr::from(A_FEW).deref(), A_FEW);
    assert_eq!(Nbstr::from_str(A_FEW).deref(), A_FEW);
    assert_eq!(Nbstr::from(A_FEW.to_string()).deref(), A_FEW);
}
#[test]
fn into() {
    let l1: &str = &Box::<str>::from(Nbstr::from("str".to_string().into_boxed_str()));
    assert_eq!(l1, "str");
    let l2: &str = &String::from(Nbstr::from("str".to_string()));
    assert_eq!(l2, "str");
    assert!(Box::<str>::from(Nbstr::from("str".to_string())) == Box::from(Nbstr::from("str")));
}
#[test]
fn from_str() {
    let mut s = String::new();
    for _ in 0..20 {// use 13 to only test stack
        // also tests for the presence of pub fn from_str
        assert_eq!(Nbstr::from_str(&s).deref(), &s);
        s.push('|');
    }
}
#[test]
fn empty() {
    let none: Option<Nbstr> = None;
    assert!(none.is_none());
    assert!(Some(Nbstr::default()).is_some());
    let empty = Nbstr::from_str("");
    assert_eq!(empty.deref(), "");
    assert!(Some(empty).is_some());
    let s1 = String::with_capacity(0);
    assert!(Some(Nbstr::from_str(&s1)).is_some());
    assert!(Some(Nbstr::from(s1)).is_some());
    let s2 = String::with_capacity(1);
    assert!(Some(Nbstr::from(s2)).is_some());
    let b = String::with_capacity(1).into_boxed_str();
    assert!(Some(Nbstr::from(b)).is_some());
}
#[test]
fn clone_from() {
    let mut s = A_FEW.to_string();
    let orig_owned = Nbstr::from(s.clone());
    let orig_literal = Nbstr::from(A_FEW);
    s = s.replace("a", "*");
    let mut a = Nbstr::from(s.clone());
    assert_eq!(a.len(), orig_owned.len());
    assert!(a != orig_owned);
    a.clone_from(&orig_owned);
    assert_eq!(a, orig_owned);
    a.clone_from(&orig_literal);
    assert_eq!(a, orig_literal);
    s.push_str("--------------------------------------------------------");
    let b = Nbstr::from(s.clone());
    a.clone_from(&b);// both boxes, but different length, no_giants was comparing one length against itself.
    assert_eq!(a, b);
}
#[test]
fn simple_derefs() {
    assert_eq!(Nbstr::from("abc").hash(&mut SipHasher::new_with_keys(0, 1)),
                           "abc".hash(&mut SipHasher::new_with_keys(0, 1))
               );
    assert_eq!(format!("a{}", Nbstr::from("bc")), format!("a{}", "bc"));
    assert_eq!(Nbstr::from("abc")==Nbstr::from_str("abc"),  "abc"=="abc");
    assert_eq!(Nbstr::from("abc")==Nbstr::from_str("aBc"),  "abc"=="aBc");
    assert_eq!(Nbstr::from("abc").cmp(&Nbstr::from_str("abc")), "abc".cmp("abc"));
    assert_eq!(Nbstr::from("abc").cmp(&Nbstr::from_str("aBc")), "abc".cmp("aBc"));
}
#[test]
#[allow(non_snake_case)]
fn is_NonZero() {
    use std::mem::{size_of,align_of};
    if cfg!(feature="unstable") {
        assert_eq!(size_of::<Option<Nbstr>>(), size_of::<Nbstr>());
    }
    assert!( size_of::<Nbstr>() < size_of::<Cow<'static, str>>() );
    assert!( size_of::<Nbstr>() % align_of::<Nbstr>() == 0 );
}
