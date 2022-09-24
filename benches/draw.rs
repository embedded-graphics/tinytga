use criterion::{criterion_group, criterion_main, Criterion};
use embedded_graphics::{
    image::Image,
    pixelcolor::{Gray8, Rgb555, Rgb888},
    prelude::*,
};
use tinytga::Tga;

// TODO: use e-g framebuffer when it's added
struct Framebuffer<C> {
    pixels: [[C; 240]; 320],
}

impl<C: PixelColor + From<Rgb888>> Framebuffer<C> {
    pub fn new() -> Self {
        let color = C::from(Rgb888::BLACK);

        Self {
            pixels: [[color; 240]; 320],
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

macro_rules! bench {
    ($c:expr, $color_type:ty, $file:expr) => {
        $c.bench_function(concat!(stringify!($color_type), " ", $file), |b| {
            let mut fb = Framebuffer::<$color_type>::new();
            b.iter(|| {
                let bmp = Tga::<$color_type>::from_slice(include_bytes!(concat!(
                    "../tests/",
                    $file,
                    ".tga"
                )))
                .unwrap();
                Image::new(&bmp, Point::zero()).draw(&mut fb).unwrap();
            })
        });
    };

    ($c:expr, $color_type:ty) => {
        bench!($c, $color_type, "logo_type1_16bpp_bl");
        bench!($c, $color_type, "logo_type1_16bpp_tl");
        bench!($c, $color_type, "logo_type1_24bpp_bl");
        bench!($c, $color_type, "logo_type1_24bpp_tl");
        bench!($c, $color_type, "logo_type2_16bpp_bl");
        bench!($c, $color_type, "logo_type2_16bpp_tl");
        bench!($c, $color_type, "logo_type2_24bpp_bl");
        bench!($c, $color_type, "logo_type2_24bpp_tl");
        bench!($c, $color_type, "logo_type3_bl");
        bench!($c, $color_type, "logo_type3_tl");
        bench!($c, $color_type, "logo_type9_16bpp_bl");
        bench!($c, $color_type, "logo_type9_16bpp_tl");
        bench!($c, $color_type, "logo_type9_24bpp_bl");
        bench!($c, $color_type, "logo_type9_24bpp_tl");
        bench!($c, $color_type, "logo_type10_16bpp_bl");
        bench!($c, $color_type, "logo_type10_16bpp_tl");
        bench!($c, $color_type, "logo_type10_24bpp_bl");
        bench!($c, $color_type, "logo_type10_24bpp_tl");
        bench!($c, $color_type, "logo_type11_bl");
        bench!($c, $color_type, "logo_type11_tl");
    };
}

fn draw_benchmarks(c: &mut Criterion) {
    bench!(c, Rgb888);
    bench!(c, Rgb555);
    bench!(c, Gray8);
}

criterion_group!(benches, draw_benchmarks);
criterion_main!(benches);
