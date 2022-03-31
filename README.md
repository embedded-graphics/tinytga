# TinyTGA

[![Build Status](https://circleci.com/gh/embedded-graphics/tinytga/tree/master.svg?style=shield)](https://circleci.com/gh/embedded-graphics/tinytga/tree/master)
[![Crates.io](https://img.shields.io/crates/v/tinytga.svg)](https://crates.io/crates/tinytga)
[![Docs.rs](https://docs.rs/tinytga/badge.svg)](https://docs.rs/tinytga)
[![embedded-graphics on Matrix](https://img.shields.io/matrix/rust-embedded-graphics:matrix.org)](https://matrix.to/#/#rust-embedded-graphics:matrix.org)

## [Documentation](https://docs.rs/tinytga)

A small TGA parser designed for use with [embedded-graphics] targeting no-std environments but
usable anywhere. Beyond parsing the image header, no other allocations are made.

tinytga provides two methods of accessing the pixel data inside a TGA file. The most convenient
way is to use a color type provided by [embedded-graphics] to define the format stored inside
the TGA file. But it is also possible to directly access the raw pixel representation instead.

## Examples

### Using `Tga` to draw an image

This example demonstrates how a TGA image can be drawn to a [embedded-graphics] draw target.

The code uses the `Tga` struct and only works if the color format inside the TGA file is known
at compile time. While this makes the code less flexible it offers the best performance by
making sure that no unnecessary color conversions are used.

```rust
use embedded_graphics::{image::Image, pixelcolor::Rgb888, prelude::*};
use tinytga::Tga;

// Include an image from a local path as bytes
let data = include_bytes!("../tests/chessboard_4px_rle.tga");

let tga: Tga<Rgb888> = Tga::from_slice(data).unwrap();

let image = Image::new(&tga, Point::zero());

image.draw(&mut display)?;
```


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
