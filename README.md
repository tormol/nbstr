# Nbstr
A lean `Cow<'static, str>` that cannot be written to.

(Name stands for *non-borrowed `str`*).

Might be useful when you want to store many short `str`s that you won't modify, and expect many of them to be literals.

Size is reduced by replacing `String` with `Box<str>`, which removes one `usize`, at the cost of some reallocation.  

To avoid boxing when possible, short `str`s can be stored inside the struct itself, replacing pointer and length. The length of the short `str` is then stored as a part of the tag/discriminant, which is why Nbstr is a struct and not an enum.  
The definition of 'short' depends on architecture and features.  


## Usage

Add
```toml
[dependencies]
nbstr = "0.9"
# or if you can use unstable features:
nbstr = {version="0.9", features=["no_giants"]}
```
to Cargo.toml, and then

```rust
extern crate nbstr;
use nbstr::Nbstr;

#[derive(Default)]
struct Container {// <- no lifetime
    list: Vec<Nbstr>
}
impl Container {
    fn append<S:Into<Nbstr>>(&mut self,  s: S) {
        self.list.push(s.into());
    }
}
fn main() {
    let mut c = Container::default();
    c.append("foo");// &'static str
    {   // &str wouldn't work here since the strings goes out of scope before the Vec
        c.append(Nbstr::from_str(&("bar".to_string())));// is short enough to avoid allocating,
        c.append("baz".to_string());
    }
    println!("{:?}", c.list);
}
```


## Feature flags

There are four variants of Nbstr, selected with cargo features:
* The default: works on stable Rust.  
  Size is 2*usize+2 without any alignment, Lacks `Option<>` optimization.

* **unstable**: Reduce struct size with the unstable features
  `#[unsafe_no_drop_flag]` and `NonZero` (which `Option<>` optimization is based on).
  Size is 2*usize+1 without any alignment.

* **no_giants**: Use the upper bits of length for the discriminant.  
  This puts a limit on how long `str`s Nbstr can store, but reduces struct size to that of `&str`.

  The limit is high enough* that is should not be an issue in most use cases of Nbstr\*\*, but since it is not enforced in release mode, it **might represent a security vulnerability**.

  Not available on stable because it depends on `#[unsafe_no_drop_flag]`.

  \* Exact size is 2^(bits - (log2(bits)-1) ) - 1, you can get it at runtime from `Nbstr::max_length()`.  
  \*\* Are megabyte long strings relevant if a lot of them are string literals?

* **64as48bit_hack**: For when you *really* want to save memory.  
  Current implementations of x86_64 have a 48 bit address space.  
  If you expect it to stay that for as long as the software is used, this flag reduces the size of Nbstr to 13 bytes.

  On other architectures, the unsafe variant (or no_giants if enabled) will be used.  
  Requires nightly rust for #[unsafe_no_drop_flag] and NonZero; If you care enough to use this hack, you care enough to use nightly.

Clippy can be enabled with **clippy**, to get a lot of warnings for things I think are OK.


## License

Apache-2.0
