use core::marker::PhantomData;
use embedded_graphics::{
    pixelcolor::{Gray8, Rgb555, Rgb888},
    prelude::*,
};

use crate::{ColorType, RawPixels};

/// Iterator over individual TGA pixels.
///
/// See the [`pixels`] method for additional information.
///
/// [`pixels`]: struct.Tga.html#method.pixels
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Pixels<'a, CI, CT> {
    raw: RawPixels<'a>,

    color_type: ColorType,

    image_color_type: PhantomData<CI>,
    target_color_type: PhantomData<CT>,
}

impl<'a, CI, CT> Pixels<'a, CI, CT>
where
    CI: PixelColor + From<<CI as PixelColor>::Raw>,
    CT: PixelColor + From<CI>,
{
    pub(crate) fn new(raw: RawPixels<'a>) -> Self {
        Self {
            raw,
            color_type: ColorType::Gray8, // not used
            image_color_type: PhantomData,
            target_color_type: PhantomData,
        }
    }
}

impl<'a, CT> Pixels<'a, Dynamic, CT>
where
    CT: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    pub(crate) fn new_dynamic(raw: RawPixels<'a>) -> Self {
        Self {
            raw,
            color_type: ColorType::Gray8, // not used
            image_color_type: PhantomData,
            target_color_type: PhantomData,
        }
    }
}

impl<CI, CT> Iterator for Pixels<'_, CI, CT>
where
    CI: PixelColor + From<<CI as PixelColor>::Raw>,
    CT: PixelColor + From<CI>,
{
    type Item = Pixel<CT>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|p| {
            let color = CI::from(CI::Raw::from_u32(p.color));

            Pixel(p.position, CT::from(color))
        })
    }
}

/// Dynamic color marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dynamic {}

impl<CT> Iterator for Pixels<'_, Dynamic, CT>
where
    CT: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888>,
{
    type Item = Pixel<CT>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|p| {
            let color = match self.color_type {
                ColorType::Gray8 => {
                    Gray8::from(<Gray8 as PixelColor>::Raw::from_u32(p.color)).into()
                }
                ColorType::Rgb555 => {
                    Rgb555::from(<Rgb555 as PixelColor>::Raw::from_u32(p.color)).into()
                }
                ColorType::Rgb888 => {
                    Rgb888::from(<Rgb888 as PixelColor>::Raw::from_u32(p.color)).into()
                }
            };

            Pixel(p.position, color)
        })
    }
}
