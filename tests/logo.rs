use embedded_graphics::{
    image::{Image, ImageRawBE, ImageRawLE},
    pixelcolor::{Gray8, Rgb555, Rgb888},
    prelude::*,
};
use tinytga::{Bpp, Compression, DataType, ImageOrigin, Tga};

const WIDTH: usize = 240;
const HEIGHT: usize = 320;

// TODO: use e-g framebuffer when it's added
#[derive(Debug, PartialEq)]
struct Framebuffer<C> {
    pixels: [[C; 240]; 320],
}

impl<C: PixelColor + From<Rgb888> + std::fmt::Debug> Framebuffer<C> {
    pub fn new() -> Self {
        let color = C::from(Rgb888::BLACK);

        Self {
            pixels: [[color; WIDTH]; HEIGHT],
        }
    }

    pub fn from_image(image: impl ImageDrawable<Color = C>) -> Self {
        let mut framebuffer = Framebuffer::<C>::new();
        Image::new(&image, Point::zero())
            .draw(&mut framebuffer)
            .unwrap();
        framebuffer
    }

    pub fn pixels(&self) -> impl Iterator<Item = C> + '_ {
        self.pixels.iter().flatten().copied()
    }

    pub fn assert_eq(&self, expected: &Self) {
        let zipped = || self.pixels().zip(expected.pixels());

        let errors = zipped().filter(|(a, b)| a != b).count();
        let first_error = zipped()
            .enumerate()
            .find(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (Point::new((i % WIDTH) as i32, (i / WIDTH) as i32), a, b));

        if self != expected {
            let first_error = first_error.unwrap();
            panic!(
                "framebuffer differs from expected\n{} errors\nfirst error at ({}): {:?} (expected {:?})",
                errors,
                first_error.0,
                first_error.1,
                first_error.2,
            );
        }
    }
}

impl<C: PixelColor> DrawTarget for Framebuffer<C> {
    type Color = C;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(p, c) in pixels {
            self.pixels[p.y as usize][p.x as usize] = c;
        }

        Ok(())
    }
}

impl<C> OriginDimensions for Framebuffer<C> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        Size::new(240, 320)
    }
}

fn expected_rgb555() -> Framebuffer<Rgb555> {
    Framebuffer::from_image(ImageRawLE::<Rgb555>::new(
        include_bytes!("logo_rgb555.raw"),
        WIDTH as u32,
    ))
}

fn expected_rgb888() -> Framebuffer<Rgb888> {
    Framebuffer::from_image(ImageRawBE::<Rgb888>::new(
        include_bytes!("logo_rgb888.raw"),
        WIDTH as u32,
    ))
}

fn expected_gray8() -> Framebuffer<Gray8> {
    Framebuffer::from_image(ImageRawBE::<Gray8>::new(
        include_bytes!("logo_gray8.raw"),
        WIDTH as u32,
    ))
}

#[track_caller]
fn assert_format<C>(
    tga: &Tga<C>,
    data_type: DataType,
    compression: Compression,
    pixel_depth: Bpp,
    image_origin: ImageOrigin,
    color_map_depth: Option<Bpp>,
) where
    C: PixelColor + From<Rgb888> + From<Rgb555> + From<Gray8>,
{
    assert_eq!(tga.as_raw().header().data_type, data_type);
    assert_eq!(tga.as_raw().header().compression, compression);
    assert_eq!(tga.as_raw().header().pixel_depth, pixel_depth);
    assert_eq!(tga.as_raw().header().image_origin, image_origin);

    assert_eq!(tga.as_raw().header().color_map_depth, color_map_depth);
    assert_eq!(tga.as_raw().header().color_map_start, 0);
    if color_map_depth.is_some() {
        assert!(tga.as_raw().header().color_map_len > 0);
    } else {
        assert_eq!(tga.as_raw().header().color_map_len, 0);
    }
}

#[test]
fn logo_type1_16bpp_tl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type1_16bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        Some(Bpp::Bits16),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type1_16bpp_bl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type1_16bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        Some(Bpp::Bits16),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type1_24bpp_tl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type1_24bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        Some(Bpp::Bits24),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type1_24bpp_bl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type1_24bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        Some(Bpp::Bits24),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type2_16bpp_tl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type2_16bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits16,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type2_16bpp_bl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type2_16bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits16,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type2_24bpp_tl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type2_24bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits24,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type2_24bpp_bl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type2_24bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits24,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type2_24bpp_tr() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type2_24bpp_tr.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits24,
        ImageOrigin::TopRight,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type2_24bpp_br() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type2_24bpp_br.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Uncompressed,
        Bpp::Bits24,
        ImageOrigin::BottomRight,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type3_tl() {
    let tga = Tga::<Gray8>::from_slice(include_bytes!("logo_type3_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::BlackAndWhite,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_gray8());
}

#[test]
fn logo_type3_bl() {
    let tga = Tga::<Gray8>::from_slice(include_bytes!("logo_type3_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::BlackAndWhite,
        Compression::Uncompressed,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_gray8());
}

#[test]
fn logo_type9_16bpp_tl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type9_16bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        Some(Bpp::Bits16),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type9_16bpp_bl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type9_16bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        Some(Bpp::Bits16),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type9_24bpp_tl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type9_24bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        Some(Bpp::Bits24),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type9_24bpp_bl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type9_24bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::ColorMapped,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        Some(Bpp::Bits24),
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type10_16bpp_tl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type10_16bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Rle,
        Bpp::Bits16,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type10_16bpp_bl() {
    let tga = Tga::<Rgb555>::from_slice(include_bytes!("logo_type10_16bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Rle,
        Bpp::Bits16,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb555());
}

#[test]
fn logo_type10_24bpp_tl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type10_24bpp_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Rle,
        Bpp::Bits24,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type10_24bpp_bl() {
    let tga = Tga::<Rgb888>::from_slice(include_bytes!("logo_type10_24bpp_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::TrueColor,
        Compression::Rle,
        Bpp::Bits24,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_rgb888());
}

#[test]
fn logo_type11_tl() {
    let tga = Tga::<Gray8>::from_slice(include_bytes!("logo_type11_tl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::BlackAndWhite,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::TopLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_gray8());
}

#[test]
fn logo_type11_bl() {
    let tga = Tga::<Gray8>::from_slice(include_bytes!("logo_type11_bl.tga")).unwrap();
    assert_format(
        &tga,
        DataType::BlackAndWhite,
        Compression::Rle,
        Bpp::Bits8,
        ImageOrigin::BottomLeft,
        None,
    );

    Framebuffer::from_image(tga).assert_eq(&expected_gray8());
}
