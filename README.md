# inno

[![CI Status](https://github.com/russellbanks/inno/workflows/CI/badge.svg)](https://github.com/russellbanks/inno/actions)
[![Latest version](https://img.shields.io/crates/v/inno.svg)](https://crates.io/crates/inno)
[![Documentation](https://docs.rs/inno/badge.svg)](https://docs.rs/inno)
![License](https://img.shields.io/crates/l/inno.svg)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
inno = "0.1"
```

## Example

```rust
use inno::Inno;

fn main() {
    let inno_file = File::new("innosetup-6.4.3.exe").unwrap();

    let inno = Inno::new(file).unwrap();

    println!("{}", inno.header.app_name);
}
```

## Acknowledgements

* [innoextract](https://github.com/dscharrer/innoextract)
* [Inno Setup](https://jrsoftware.org/isinfo.php)

## License

Licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE.md) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT.md) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
