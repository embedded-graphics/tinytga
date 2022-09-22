use core::{convert::TryInto, marker::PhantomData};

use embedded_graphics::{
    pixelcolor::raw::{RawU16, RawU24, RawU32, RawU8},
    prelude::*,
};

use crate::{raw_tga::RawTga, Bpp, Compression};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Uncompressed {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rle {}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RawColors<'a, R, F> {
    remaining_data: &'a [u8],

    rle_pixel: u32,
    rle_repeat: u8,
    rle_take_raw: u8,

    raw_data_type: PhantomData<R>,
    format: PhantomData<F>,
}

impl<'a, R: RawData, F> RawColors<'a, R, F> {
    pub fn new(raw_tga: &'a RawTga<'a>) -> Self {
        debug_assert_eq!(
            usize::from(raw_tga.image_data_bpp().bits()),
            R::BITS_PER_PIXEL
        );

        let image_data = raw_tga.image_data();

        Self {
            remaining_data: image_data,
            rle_pixel: 0,
            rle_repeat: 0,
            rle_take_raw: 0,
            raw_data_type: PhantomData,
            format: PhantomData,
        }
    }
}

trait NextColor<R> {
    fn next_color(&mut self) -> Option<R>;
}

impl<'a, F> NextColor<RawU8> for RawColors<'a, RawU8, F> {
    fn next_color(&mut self) -> Option<RawU8> {
        self.remaining_data.split_first().map(|(r, rest)| {
            self.remaining_data = rest;
            RawU8::new(*r)
        })
    }
}

impl<'a, F> NextColor<RawU16> for RawColors<'a, RawU16, F> {
    fn next_color(&mut self) -> Option<RawU16> {
        self.remaining_data.get(0..2).map(|bytes| {
            let bytes: [u8; 2] = bytes.try_into().unwrap();

            self.remaining_data = &self.remaining_data[2..];

            RawU16::new(u16::from_le_bytes(bytes))
        })
    }
}

impl<'a, F> NextColor<RawU24> for RawColors<'a, RawU24, F> {
    fn next_color(&mut self) -> Option<RawU24> {
        self.remaining_data.get(0..3).map(|bytes| {
            let mut bytes2 = [0u8; 4];
            bytes2[0..3].copy_from_slice(bytes);

            self.remaining_data = &self.remaining_data[3..];

            RawU24::new(u32::from_le_bytes(bytes2))
        })
    }
}

impl<'a, F> NextColor<RawU32> for RawColors<'a, RawU32, F> {
    fn next_color(&mut self) -> Option<RawU32> {
        self.remaining_data.get(0..4).map(|bytes| {
            let bytes: [u8; 4] = bytes.try_into().unwrap();

            self.remaining_data = &self.remaining_data[4..];

            RawU32::new(u32::from_le_bytes(bytes))
        })
    }
}

impl<'a, R> Iterator for RawColors<'a, R, Uncompressed>
where
    Self: NextColor<R>,
    R: RawData,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_color().or_else(|| Some(R::from_u32(0)))
    }
}

impl<'a, R> Iterator for RawColors<'a, R, Rle>
where
    Self: NextColor<R>,
    R: RawData,
    R::Storage: Into<u32>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.rle_repeat > 0 {
                self.rle_repeat -= 1;
                break Some(R::from_u32(self.rle_pixel));
            } else if self.rle_take_raw > 0 {
                self.rle_take_raw -= 1;
                break self.next_color();
            } else {
                let (type_and_count, rest) = self.remaining_data.split_first()?;
                self.remaining_data = rest;

                // The pixel count is encoded in the lower 7 bits and the actual number of pixels
                // is one more than the value stored in the packet.
                let pixel_count = (*type_and_count & 0x7F) + 1;

                // The packet type is encoded in the upper bit: 0 -> Raw, 1 -> Rle
                if *type_and_count & 0x80 != 0 {
                    let pixel = self.next_color()?;

                    self.rle_repeat = pixel_count;
                    self.rle_pixel = pixel.into_inner().into();
                } else {
                    self.rle_take_raw = pixel_count;
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum DynamicRawColors<'a> {
    Bpp8Uncompressed(RawColors<'a, RawU8, Uncompressed>),
    Bpp8Rle(RawColors<'a, RawU8, Rle>),
    Bpp16Uncompressed(RawColors<'a, RawU16, Uncompressed>),
    Bpp16Rle(RawColors<'a, RawU16, Rle>),
    Bpp24Uncompressed(RawColors<'a, RawU24, Uncompressed>),
    Bpp24Rle(RawColors<'a, RawU24, Rle>),
    Bpp32Uncompressed(RawColors<'a, RawU32, Uncompressed>),
    Bpp32Rle(RawColors<'a, RawU32, Rle>),
}

/// Iterator over individual TGA pixels.
///
/// See the [`pixels`] method for additional information.
///
/// [`pixels`]: struct.RawTga.html#method.pixels
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RawPixels<'a> {
    raw_tga: &'a RawTga<'a>,
    colors: DynamicRawColors<'a>,
    position: Point,
}

impl<'a> RawPixels<'a> {
    pub(crate) fn new(raw_tga: &'a RawTga<'a>) -> Self {
        let colors = match (raw_tga.image_data_bpp(), raw_tga.compression()) {
            (Bpp::Bits8, Compression::Uncompressed) => {
                DynamicRawColors::Bpp8Uncompressed(RawColors::new(raw_tga))
            }
            (Bpp::Bits8, Compression::Rle) => DynamicRawColors::Bpp8Rle(RawColors::new(raw_tga)),
            (Bpp::Bits16, Compression::Uncompressed) => {
                DynamicRawColors::Bpp16Uncompressed(RawColors::new(raw_tga))
            }
            (Bpp::Bits16, Compression::Rle) => DynamicRawColors::Bpp16Rle(RawColors::new(raw_tga)),
            (Bpp::Bits24, Compression::Uncompressed) => {
                DynamicRawColors::Bpp24Uncompressed(RawColors::new(raw_tga))
            }
            (Bpp::Bits24, Compression::Rle) => DynamicRawColors::Bpp24Rle(RawColors::new(raw_tga)),
            (Bpp::Bits32, Compression::Uncompressed) => {
                DynamicRawColors::Bpp32Uncompressed(RawColors::new(raw_tga))
            }
            (Bpp::Bits32, Compression::Rle) => DynamicRawColors::Bpp32Rle(RawColors::new(raw_tga)),
        };

        let start_y = if raw_tga.image_origin().is_bottom() {
            raw_tga.size().height.saturating_sub(1)
        } else {
            0
        };

        Self {
            raw_tga,
            colors,
            position: Point::new(0, start_y as i32),
        }
    }

    /// Returns the next pixel position.
    fn next_position(&mut self) -> Option<Point> {
        if self.position.y < 0 || self.position.y >= self.raw_tga.size().height as i32 {
            return None;
        }

        let position = self.position;

        self.position.x += 1;

        if self.position.x >= self.raw_tga.size().width as i32 {
            self.position.x = 0;

            if self.raw_tga.image_origin().is_bottom() {
                self.position.y -= 1;
            } else {
                self.position.y += 1;
            }
        }

        Some(position)
    }
}

impl Iterator for RawPixels<'_> {
    type Item = RawPixel;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.next_position()?;

        let color = match &mut self.colors {
            DynamicRawColors::Bpp8Uncompressed(colors) => u32::from(colors.next()?.into_inner()),
            DynamicRawColors::Bpp8Rle(colors) => u32::from(colors.next()?.into_inner()),
            DynamicRawColors::Bpp16Uncompressed(colors) => u32::from(colors.next()?.into_inner()),
            DynamicRawColors::Bpp16Rle(colors) => u32::from(colors.next()?.into_inner()),
            DynamicRawColors::Bpp24Uncompressed(colors) => colors.next()?.into_inner(),
            DynamicRawColors::Bpp24Rle(colors) => colors.next()?.into_inner(),
            DynamicRawColors::Bpp32Uncompressed(colors) => colors.next()?.into_inner(),
            DynamicRawColors::Bpp32Rle(colors) => colors.next()?.into_inner(),
        };

        Some(RawPixel::new(position, color))
    }
}

/// Pixel with raw pixel color.
///
/// This struct is returned by the [`RawPixels`] iterator.
///
/// [`RawPixels`]: struct.RawPixels.html
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct RawPixel {
    /// The position relative to the top left corner of the image.
    pub position: Point,

    /// The raw pixel color.
    pub color: u32,
}

impl RawPixel {
    /// Creates a new raw pixel.
    pub const fn new(position: Point, color: u32) -> Self {
        Self { position, color }
    }
}
