use embedded_graphics::{
    image::Image,
    mock_display::{ColorMapping, MockDisplay},
    pixelcolor::{Gray8, Rgb555, Rgb888},
    prelude::*,
};
use paste::paste;
use tinytga::Tga;

const CHESSBOARD_PATTERN: &[&str] = &[
    "WKWK", //
    "KRKG", //
    "WKBK", //
    "KWKW", //
];

const GRAY_PATTERN: &[&str] = &[
    "0F0F0F0F0",
    "00FF00FF0",
    "0000FFFF0",
    "012345670",
    "89ABCDEF0",
];

const COLOR_PATTERN: &[&str] = &[
    "WKRGBYMCW",
    "KKRGBYMCW",
    "WKRGBYMCW",
    "KKKKKKKKK",
    "WKWCMYBGR",
];

#[test]
fn chessboard_compressed() {
    let tga = Tga::from_slice(include_bytes!("./chessboard_4px_rle.tga")).unwrap();
    let image = Image::new(&tga, Point::zero());

    let mut display = MockDisplay::<Rgb888>::new();
    image.draw(&mut display).unwrap();

    display.assert_pattern(CHESSBOARD_PATTERN);
}

#[test]
fn chessboard_uncompressed() {
    let tga = Tga::from_slice(include_bytes!("./chessboard_raw.tga")).unwrap();
    let image = Image::new(&tga, Point::zero());

    let mut display = MockDisplay::<Rgb888>::new();
    image.draw(&mut display).unwrap();

    display.assert_pattern(CHESSBOARD_PATTERN);
}

fn test_tga<C>(data: &[u8], pattern: &[&str])
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888> + ColorMapping,
{
    let tga = Tga::from_slice(data).unwrap();
    let image = Image::new(&tga, Point::zero());

    let mut display = MockDisplay::<C>::new();
    image.draw(&mut display).unwrap();

    display.assert_pattern(pattern);
}

fn test_pixels_iter<C>(data: &[u8], pattern: &[&str])
where
    C: PixelColor + From<Gray8> + From<Rgb555> + From<Rgb888> + ColorMapping,
{
    let tga = Tga::from_slice(data).unwrap();

    let mut display = MockDisplay::<Rgb888>::new();
    tga.pixels().draw(&mut display).unwrap();

    let expected: MockDisplay<Rgb888> = MockDisplay::<C>::from_pattern(pattern).map(|c| c.into());

    display.assert_eq(&expected);
}

macro_rules! test_tga {
    ($image_type:ident, $color_type:ty, $pattern:expr) => {
        paste! {
            #[test]
            fn [<$image_type _bl>]() {
                test_tga::<$color_type>(include_bytes!(concat!(stringify!($image_type), "_bl.tga")), $pattern);
            }

            #[test]
            fn [<$image_type _tl>]() {
                test_tga::<$color_type>(include_bytes!(concat!(stringify!($image_type), "_tl.tga")), $pattern);
            }

            #[test]
            fn [<$image_type _bl_pixels_iter>]() {
                test_pixels_iter::<$color_type>(include_bytes!(concat!(stringify!($image_type), "_bl.tga")), $pattern);
            }

            #[test]
            fn [<$image_type _tl_pixels_iter>]() {
                test_pixels_iter::<$color_type>(include_bytes!(concat!(stringify!($image_type), "_tl.tga")), $pattern);
            }
        }
    };

    ($image_type:ident, Rgb555) => {
        test_tga!($image_type, Rgb555, COLOR_PATTERN);
    };

    ($image_type:ident, Rgb888) => {
        test_tga!($image_type, Rgb888, COLOR_PATTERN);
    };

    ($image_type:ident, Gray8) => {
        test_tga!($image_type, Gray8, GRAY_PATTERN);
    };
}

// Type 1: color mapped, uncompressed
test_tga!(type1_16bpp, Rgb555);
test_tga!(type1_24bpp, Rgb888);

// Type 2: true color, uncompressed
test_tga!(type2_16bpp, Rgb555);
test_tga!(type2_24bpp, Rgb888);

// Type 3: grayscale, uncompressed
test_tga!(type3, Gray8);

// Type 9: color mapped, RLE compressed
test_tga!(type9_16bpp, Rgb555);
test_tga!(type9_24bpp, Rgb888);

// Type 10: true color, RLE compressed
test_tga!(type10_16bpp, Rgb555);
test_tga!(type10_24bpp, Rgb888);

// Type 11: grayscale, RLE compressed
test_tga!(type11, Gray8);
