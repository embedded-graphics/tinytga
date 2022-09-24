use embedded_graphics::{
    pixelcolor::{
        raw::{RawU16, RawU24, RawU8},
        Gray8, Rgb555, Rgb888,
    },
    prelude::*,
};

use crate::{ColorType, RawPixel, RawPixels, Tga};

/// Iterator over individual TGA pixels.
///
/// See the [`pixels`] method for additional information.
///
/// [`pixels`]: struct.Tga.html#method.pixels
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Pixels<'a, C> {
    tga: &'a Tga<'a, C>,
    raw_pixels: RawPixels<'a>,
}

impl<'a, C> Pixels<'a, C>
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    pub(crate) fn new(tga: &'a Tga<'a, C>) -> Self {
        Self {
            tga,
            raw_pixels: RawPixels::new(&tga.raw),
        }
    }
}

impl<C> Iterator for Pixels<'_, C>
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    type Item = Pixel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let RawPixel {
            position,
            mut color,
        } = self.raw_pixels.next()?;

        if let Some(color_map) = self.tga.raw.color_map() {
            color = color_map.get_raw(color as usize).unwrap()
        }

        let color = match self.tga.image_color_type {
            ColorType::Gray8 => Gray8::from(RawU8::from_u32(color)).into(),
            ColorType::Rgb555 => Rgb555::from(RawU16::from_u32(color)).into(),
            ColorType::Rgb888 => Rgb888::from(RawU24::from_u32(color)).into(),
        };

        Some(Pixel(position, color))
    }
}
