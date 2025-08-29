![Build Status](https://github.com/jaredforth/webp/actions/workflows/rust.yml/badge.svg)
[![Crate](https://img.shields.io/crates/v/webp.svg)](https://crates.io/crates/webp)
[![API](https://docs.rs/webp/badge.svg)](https://docs.rs/webp)
![Crates.io](https://img.shields.io/crates/d/webp)

# webp

A WebP conversion library

Documentation:

- [API Reference](https://docs.rs/webp)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
webp = "0.3"
```

## Examples

An example for converting an image between JPEG and WebP formats is provided in the
`examples` directory. It can be run using

```sh
cargo run --release --example convert
```

## License

**webp** is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.

The photo `lake.jpg` included in the `assets/` directory is licensed under
[CC0](https://creativecommons.org/publicdomain/zero/1.0/)/"Public Domain Dedication".
