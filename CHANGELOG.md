# Changelog

[`tinytga`](https://crates.io/crates/tinytga) is a no_std, low memory footprint TGA loading library for embedded applications.

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Added support for bottom right and top right image origins.

### Changed

- **(breaking)** [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Use 1.61 as MSRV.
- **(breaking)** [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Replaced `ImageType` enum with `DataType` and `Compression`.
- **(breaking)** [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Color types used with `Tga` are now required to implement `From<Gray8> + From<Rgb555> + From<Rgb888>`.
- [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Improved drawing performance for bottom left origin images by using `fill_contiguous`.
- [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Use correct lifetimes for `RawTga::image_id`, `RawTga::developer_dictionary` and `RawTga::extension_area`.

### Removed

- **(breaking)** [#16](https://github.com/embedded-graphics/tinybmp/pull/16) Removed `DynamicTga`, use `Tga` instead.

## [0.4.1] - 2021-06-16

### Changed

- [#10](https://github.com/embedded-graphics/tinybmp/pull/10) Bump embedded-graphics minimum version from 0.7.0 to 0.7.1

## [0.4.0] - 2021-06-06

## [0.4.0-beta.1] - 2021-05-24

## [0.4.0-alpha.1] - 2020-12-27

### Changed

- **(breaking)** [#3](https://github.com/embedded-graphics/tinytga/pull/3) `tinytga` now depends on `embedded-graphics-core` instead of `embedded-graphics`.

## [0.4.0-alpha.1 - `embedded-graphics` repository] - 2020-12-27

> Note: PR numbers from this point onwards are from the old `embedded-graphics/embedded-graphics` repository. New PR numbers above this note refer to PRs in the `embedded-graphics/tinytga` repository.

### Changed

- **(breaking)** [#407](https://github.com/embedded-graphics/embedded-graphics/pull/407) The `image_descriptor` in `TgaHeader` was replaced by `image_origin` and `alpha_channel_bits`.
- **(breaking)** [#420](https://github.com/embedded-graphics/embedded-graphics/pull/420) To support the new embedded-graphics 0.7 image API a color type parameter was added to `Tga`.
- **(breaking)** [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) The `graphics` feature was removed and the `embedded-graphics` dependency is now non optional.
- **(breaking)** [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) `Tga` no longer implements `IntoIterator`. Pixel iterators can now be created using the `pixels` and `raw_pixels` methods.
- **(breaking)** [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) `Tga::from_slice` now checks that the specified color type matches the bit depth of the image.
- **(breaking)** [#450](https://github.com/embedded-graphics/embedded-graphics/pull/450) The `TgaFooter` struct was replaced by the `developer_dictionary` and `extension_area` methods in `RawTga`.
- **(breaking)** [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) `Tga::width` and `Tga::height` were replaced by `Tga::size` which requires `embedded_graphics::geometry::OriginDimensions` to be in scope (also included in the embedded-graphics `prelude`).
- **(breaking)** [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) The color map can now be accessed using the new `ColorMap` type.
- **(breaking)** [#450](https://github.com/embedded-graphics/embedded-graphics/pull/450) `Tga` no longer provides direct access to low level information like the TGA header, instead `Tga::as_raw` can be used to access the underlying `RawTga` instance.

### Added

- [#407](https://github.com/embedded-graphics/embedded-graphics/pull/407) Added support for bottom-left origin images to `TgaIterator`.
- [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) The image ID can now be accessed using `Tga::image_id`.
- [#450](https://github.com/embedded-graphics/embedded-graphics/pull/450) Added `RawTga` to use `tinytga` without using a embedded-graphic color type.
- [#450](https://github.com/embedded-graphics/embedded-graphics/pull/450) Added `Tga::from_raw` to convert a `RawTga` into a `Tga` object.
- [#450](https://github.com/embedded-graphics/embedded-graphics/pull/450) Added `DynamicTga` to allow drawing of TGA images without a known color format at compile time.

### Fixed

- [#407](https://github.com/embedded-graphics/embedded-graphics/pull/407) Additional data in `pixel_data`, beyond `width * height` pixels, is now discarded by `TgaIterator`.
- [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) Images with unsupported BPP values in the header no longer cause panics. Instead an error is returned by `Tga::from_slice`.
- [#430](https://github.com/embedded-graphics/embedded-graphics/pull/430) Errors during the execution of a pixel iterator no longer cause panics. Instead the corrupted portion of the image is filled with black pixels.

## [0.3.2] - 2020-03-20

## [0.3.1] - 2020-02-17

- **(breaking)** [#247](https://github.com/embedded-graphics/embedded-graphics/pull/247) "reverse" integration of tinytga into [`embedded-graphics`](https://crates.io/crates/embedded-graphics). tinytga now has a `graphics` feature that must be turned on to enable embedded-graphics support. The `tga` feature from embedded-graphics is removed.

  **Before**

  `Cargo.toml`

  ```toml
  [dependencies]
  embedded-graphics = { version = "0.6.0-alpha.3", features = [ "tga" ]}
  ```

  Your code

  ```rust
  use embedded_graphics::prelude::*;
  use embedded_graphics::image::ImageTga;

  let image = ImageTga::new(include_bytes!("../../../assets/patch.tga")).unwrap();
  display.draw(&image);
  ```

  **After**

  `Cargo.toml`

  ```toml
  [dependencies]
  embedded-graphics = "0.6.0"
  tinytga = { version = "*", features = [ "graphics" ]}
  ```

  Your code

  ```rust
  use embedded_graphics::{prelude::*, image::Image};
  use tinytga::Tga;

  let image = Tga::new(include_bytes!("../../../assets/patch.tga")).unwrap();
  let image = Image::new(&image);
  display.draw(&image);
  ```

## 0.2.0

### Added

- [#217](https://github.com/embedded-graphics/embedded-graphics/pull/217) Added support for TGA files with color map.

### Fixed

- [#217](https://github.com/embedded-graphics/embedded-graphics/pull/217) Images without a TGA footer are now parsed correctly.
- [#216](https://github.com/embedded-graphics/embedded-graphics/pull/216) Fixed integer overflow for some RLE compressed TGA files.
- [#218](https://github.com/embedded-graphics/embedded-graphics/pull/218) Test README examples in CI and update them to work with latest crate versions.

<!-- next-url -->
[unreleased]: https://github.com/embedded-graphics/tinytga/compare/v0.4.1...HEAD

[0.4.1]: https://github.com/embedded-graphics/tinytga/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/embedded-graphics/tinytga/compare/v0.4.0-beta.1...v0.4.0
[0.4.0-beta.1]: https://github.com/embedded-graphics/tinytga/compare/v0.4.0-alpha.1...v0.4.0-beta.1
[0.4.0-alpha.1]: https://github.com/embedded-graphics/tinytga/compare/after-split...v0.4.0-alpha.1
[0.4.0-alpha.1 - `embedded-graphics` repository]: https://github.com/embedded-graphics/embedded-graphics/compare/tinytga-v0.3.2...before-split
[0.3.2]: https://github.com/embedded-graphics/embedded-graphics/compare/tinytga-v0.3.0...tinytga-v0.3.2
[0.3.1]: https://github.com/embedded-graphics/embedded-graphics/compare/tinytga-v0.2.0...tinytga-v0.3.1
