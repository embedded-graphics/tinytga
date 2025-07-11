//! A small TGA parser designed for use with [embedded-graphics] targeting no-std environments but
//! usable anywhere. Beyond parsing the image header, no other allocations are made.
//!
//! tinytga provides two methods of accessing the pixel data inside a TGA file. The most convenient
//! way is to use a color type provided by [embedded-graphics] to define the format stored inside
//! the TGA file. But it is also possible to directly access the raw pixel representation instead.
//!
//! # Examples
//!
//! ## Using `Tga` to draw an image
//!
//! This example demonstrates how a TGA image can be drawn to a [embedded-graphics] draw target.
//!
//! ```rust
//! # fn main() -> Result<(), core::convert::Infallible> {
//! # let mut display = embedded_graphics::mock_display::MockDisplay::default();
//! use embedded_graphics::{image::Image, pixelcolor::Rgb888, prelude::*};
//! use tinytga::Tga;
//!
//! // Include an image from a local path as bytes
//! let data = include_bytes!("../tests/chessboard_4px_rle.tga");
//!
//! let tga: Tga<Rgb888> = Tga::from_slice(data).unwrap();
//!
//! let image = Image::new(&tga, Point::zero());
//!
//! image.draw(&mut display)?;
//! # Ok::<(), core::convert::Infallible>(()) }
//! ```
//!
//! ## Accessing pixels using an embedded-graphics color type
//!
//! If [embedded-graphics] is not used to draw the TGA image, the color types provided by
//! [embedded-graphics] can still be used to access the pixel data using the
//! [`pixels`](struct.Tga.html#method.pixels) method.
//!
//! ```rust
//! use embedded_graphics::{prelude::*, pixelcolor::Rgb888};
//! use tinytga::Tga;
//!
//! // Include an image from a local path as bytes
//! let data = include_bytes!("../tests/chessboard_4px_rle.tga");
//!
//! // Create a TGA instance from a byte slice.
//! // The color type is set by defining the type of the `img` variable.
//! let img: Tga<Rgb888> = Tga::from_slice(data).unwrap();
//!
//! // Check the size of the image.
//! assert_eq!(img.size(), Size::new(4, 4));
//!
//! // Collect pixels into a vector.
//! let pixels: Vec<_> = img.pixels().collect();
//! ```
//!
//! ## Accessing raw pixel data
//!
//! If [embedded-graphics] is not used in the target application, the raw image data can be
//! accessed with the [`pixels`](struct.RawTga.html#method.pixels) method on
//! [`RawTga`]. The returned iterator produces a `u32` for each pixel value.
//!
//! ```rust
//! use embedded_graphics::{prelude::*, pixelcolor::Rgb888};
//! use tinytga::{Bpp, Compression, DataType, ImageOrigin, RawPixel, RawTga, TgaHeader};
//!
//! // Include an image from a local path as bytes.
//! let data = include_bytes!("../tests/chessboard_4px_rle.tga");
//!
//! // Create a TGA instance from a byte slice.
//! let img = RawTga::from_slice(data).unwrap();
//!
//! // Take a look at the raw image header.
//! assert_eq!(
//!     img.header(),
//!     TgaHeader {
//!         id_len: 0,
//!         has_color_map: false,
//!         data_type: DataType::TrueColor,
//!         compression: Compression::Rle,
//!         color_map_start: 0,
//!         color_map_len: 0,
//!         color_map_depth: None,
//!         x_origin: 0,
//!         y_origin: 4,
//!         width: 4,
//!         height: 4,
//!         pixel_depth: Bpp::Bits24,
//!         image_origin: ImageOrigin::TopLeft,
//!         alpha_channel_depth: 0,
//!     }
//! );
//!
//! // Collect raw pixels into a vector.
//! let pixels: Vec<_> = img.pixels().collect();
//! ```
//!
//! # Embedded-graphics drawing performance
//!
//! `tinytga` uses different code paths to draw images with different [`ImageOrigin`]s.
//! The performance difference between the origins will depend on the display driver, but using
//! images with the origin at the top left corner will generally result in the best performance.
//!
//! # Minimum supported Rust version
//!
//! The minimum supported Rust version for tinytga is `1.81` or greater.
//! Ensure you have the correct version of Rust installed, preferably through <https://rustup.rs>.
//!
//! [`ImageOrigin`]: enum.ImageOrigin.html
//! [embedded-graphics]: https://docs.rs/embedded-graphics
//! [`Tga`]: ./struct.Tga.html
//! [`RawTga`]: ./struct.RawTga.html

#![no_std]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]

mod color_map;
mod footer;
mod header;
mod parse_error;
mod pixels;
mod raw_iter;
mod raw_tga;

use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::{
        raw::{RawU16, RawU24, RawU8},
        Gray8, Rgb555, Rgb888,
    },
    prelude::*,
    primitives::Rectangle,
};
use raw_iter::{RawColors, Rle, Uncompressed};

pub use crate::{
    color_map::ColorMap,
    header::{Bpp, Compression, DataType, ImageOrigin, TgaHeader},
    parse_error::ParseError,
    pixels::Pixels,
    raw_iter::{RawPixel, RawPixels},
    raw_tga::RawTga,
};

/// TGA image.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tga<'a, C> {
    /// Raw TGA file.
    raw: RawTga<'a>,

    image_color_type: ColorType,

    /// Color type.
    target_color_type: PhantomData<C>,
}

impl<'a, C> Tga<'a, C>
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    /// Parses a TGA image from a byte slice.
    pub fn from_slice(data: &'a [u8]) -> Result<Self, ParseError> {
        let raw = RawTga::from_slice(data)?;

        let image_color_type = match (raw.color_bpp(), raw.data_type()) {
            (Bpp::Bits8, DataType::BlackAndWhite) => ColorType::Gray8,
            (Bpp::Bits16, DataType::ColorMapped) => ColorType::Rgb555,
            (Bpp::Bits16, DataType::TrueColor) => ColorType::Rgb555,
            (Bpp::Bits24, DataType::ColorMapped) => ColorType::Rgb888,
            (Bpp::Bits24, DataType::TrueColor) => ColorType::Rgb888,
            _ => {
                return Err(ParseError::UnsupportedTgaType(
                    raw.data_type(),
                    raw.color_bpp(),
                ));
            }
        };

        Ok(Tga {
            raw,
            image_color_type,
            target_color_type: PhantomData,
        })
    }

    /// Returns a reference to the raw TGA image.
    ///
    /// The [`RawTga`] object can be used to access lower level details about the TGA file.
    ///
    /// [`RawTga`]: struct.RawTga.html
    pub fn as_raw(&self) -> &RawTga<'a> {
        &self.raw
    }

    /// Returns an iterator over the pixels in this image.
    pub fn pixels(&self) -> Pixels<'_, C> {
        Pixels::new(self)
    }

    fn draw_colors<D>(
        &self,
        target: &mut D,
        mut colors: impl Iterator<Item = C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let bounding_box = self.bounding_box();
        if bounding_box.is_zero_sized() {
            return Ok(());
        }

        let origin = self.raw.image_origin();

        // TGA files with the origin in the top left corner can be drawn using `fill_contiguous`.
        // All other origins are drawn by falling back to `draw_iter`.
        match origin {
            ImageOrigin::TopLeft => target.fill_contiguous(&bounding_box, colors),
            ImageOrigin::BottomLeft => {
                let mut row_rect =
                    Rectangle::new(Point::zero(), Size::new(bounding_box.size.width, 1));

                for y in bounding_box.rows().rev() {
                    row_rect.top_left.y = y;
                    let row_colors = (&mut colors).take(bounding_box.size.width as usize);
                    target.fill_contiguous(&row_rect, row_colors)?;
                }

                Ok(())
            }
            ImageOrigin::TopRight => {
                let max_x = bounding_box.bottom_right().map(|p| p.x).unwrap_or_default();

                bounding_box
                    .points()
                    .zip(colors)
                    .map(|(p, c)| Pixel(Point::new(max_x - p.x, p.y), c))
                    .draw(target)
            }
            ImageOrigin::BottomRight => {
                let bottom_right = bounding_box.bottom_right().unwrap_or_default();

                bounding_box
                    .points()
                    .zip(colors)
                    .map(|(p, c)| Pixel(bottom_right - p, c))
                    .draw(target)
            }
        }
    }

    fn draw_regular<D, CI, F>(
        &self,
        target: &mut D,
        colors: RawColors<'a, CI::Raw, F>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        CI: PixelColor + From<CI::Raw> + Into<C>,
        RawColors<'a, CI::Raw, F>: Iterator<Item = CI::Raw>,
    {
        self.draw_colors(target, colors.map(|c| CI::from(c).into()))
    }

    fn draw_color_mapped<D, R, F>(
        &self,
        target: &mut D,
        indices: RawColors<'a, R, F>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        R: RawData,
        R::Storage: Into<u32>,
        RawColors<'a, R, F>: Iterator<Item = R>,
    {
        let color_map = if let Some(color_map) = self.raw.color_map() {
            color_map
        } else {
            return Ok(());
        };

        match self.image_color_type {
            ColorType::Rgb555 => {
                let colors = indices.map(|index| {
                    let index = index.into_inner().into() as usize;
                    color_map.get::<Rgb555>(index).unwrap().into()
                });

                self.draw_colors(target, colors)
            }
            ColorType::Rgb888 => {
                let colors = indices.map(|index| {
                    let index = index.into_inner().into() as usize;
                    color_map.get::<Rgb888>(index).unwrap().into()
                });

                self.draw_colors(target, colors)
            }
            // Color mapped Gray8 images aren't supported.  Using a color map for Gray8 images
            // doesn't make sense, because this encoding will always be larger than a type 3 image.
            ColorType::Gray8 => Ok(()),
        }
    }
}

impl<C> OriginDimensions for Tga<'_, C> {
    fn size(&self) -> Size {
        self.raw.size()
    }
}

impl<C> ImageDrawable for Tga<'_, C>
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match self.raw.image_data_bpp() {
            Bpp::Bits8 => match self.raw.compression() {
                Compression::Uncompressed => {
                    let colors = RawColors::<RawU8, Uncompressed>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Gray8, _>(target, colors)
                    }
                }
                Compression::Rle => {
                    let colors = RawColors::<RawU8, Rle>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Gray8, _>(target, colors)
                    }
                }
            },
            Bpp::Bits16 => match self.raw.compression() {
                Compression::Uncompressed => {
                    let colors = RawColors::<RawU16, Uncompressed>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Rgb555, _>(target, colors)
                    }
                }
                Compression::Rle => {
                    let colors = RawColors::<RawU16, Rle>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Rgb555, _>(target, colors)
                    }
                }
            },
            Bpp::Bits24 => match self.raw.compression() {
                Compression::Uncompressed => {
                    let colors = RawColors::<RawU24, Uncompressed>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Rgb888, _>(target, colors)
                    }
                }
                Compression::Rle => {
                    let colors = RawColors::<RawU24, Rle>::new(&self.raw);

                    if self.raw.color_map().is_some() {
                        self.draw_color_mapped(target, colors)
                    } else {
                        self.draw_regular::<_, Rgb888, _>(target, colors)
                    }
                }
            },
            Bpp::Bits32 => Ok(()),
        }
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.draw(&mut target.translated(-area.top_left).clipped(area))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum ColorType {
    Gray8,
    Rgb555,
    Rgb888,
}
