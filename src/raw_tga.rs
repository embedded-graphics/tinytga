use embedded_graphics::prelude::*;
use nom::{bytes::complete::take, IResult};

use crate::{
    color_map::ColorMap,
    footer::TgaFooter,
    header::{Bpp, ImageOrigin, TgaHeader},
    parse_error::ParseError,
    raw_iter::RawPixels,
    Compression, DataType,
};

/// Raw TGA image.
///
/// `RawTga` can be used to access lower level information about a TGA file and to access the
/// raw pixel data. It can be created directly by using the [`from_slice`] constructor or accessed
/// by calling [`as_raw`] method of a [`Tga`] object.
///
/// [`from_slice`]: #method.from_slice
/// [`Tga`]: struct.Tga.html
/// [`as_raw`]: struct.Tga.html#method.as_raw
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RawTga<'a> {
    /// Image data
    data: &'a [u8],

    /// Color map
    color_map: Option<ColorMap<'a>>,

    /// Image pixel data
    pixel_data: &'a [u8],

    /// Image size
    size: Size,

    /// Data type
    data_type: DataType,

    /// Compression
    compression: Compression,

    /// Bits per pixel
    bpp: Bpp,

    /// Image origin
    image_origin: ImageOrigin,
}

impl<'a> RawTga<'a> {
    /// Parse a TGA image from a byte slice.
    pub fn from_slice(data: &'a [u8]) -> Result<Self, ParseError> {
        let input = data;
        let (input, header) = TgaHeader::parse(input).map_err(|_| ParseError::Header)?;
        let (input, _image_id) = parse_image_id(input, &header).map_err(|_| ParseError::Header)?;
        let (input, color_map) = ColorMap::parse(input, &header)?;

        let footer_length = TgaFooter::parse(data).map_or(0, |footer| footer.length(data));

        // Use saturating_sub to make sure this can't panic
        let pixel_data = &input[0..input.len().saturating_sub(footer_length)];

        let size = Size::new(u32::from(header.width), u32::from(header.height));

        Ok(Self {
            data,
            color_map,
            pixel_data,
            size,
            bpp: header.pixel_depth,
            image_origin: header.image_origin,
            data_type: header.data_type,
            compression: header.compression,
        })
    }

    /// Returns the dimensions of this image.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the color map.
    ///
    /// `None` is returned if the image contains no color map.
    pub fn color_map(&self) -> Option<&ColorMap<'a>> {
        self.color_map.as_ref()
    }

    /// Returns the color bit depth (BPP) of this image.
    ///
    /// This function always returns the bit depth of the decoded pixels, regardless of how they are
    /// stored in the TGA file. Use [`image_data_bpp`] to get the number of bits used to store one
    /// pixel in the image data.
    ///
    /// [`image_data_bpp`]: #method.image_data_bpp
    pub fn color_bpp(&self) -> Bpp {
        if let Some(color_map) = &self.color_map {
            color_map.entry_bpp()
        } else {
            self.bpp
        }
    }

    /// Returns the image origin.
    pub fn image_origin(&self) -> ImageOrigin {
        self.image_origin
    }

    /// Returns the data type.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Returns the compression type.
    pub fn compression(&self) -> Compression {
        self.compression
    }

    /// Returns the raw image data contained in this image.
    pub fn image_data(&self) -> &'a [u8] {
        self.pixel_data
    }

    /// Returns the size of a single pixel in bits.
    ///
    /// This function returns the number of bits used to store a single pixel in the image data.
    ///
    /// For true color and grayscale images, where the colors are stored directly in the image data,
    /// the returned value will match the value returned by [`color_bpp`].
    ///
    /// For color mapped images, where the image data consists of color indices, the returned value
    /// describes the bit depth of the indices and may differ from the depth returned by
    /// [`color_bpp`].
    ///
    /// [`color_bpp`]: #method.color_bpp
    pub fn image_data_bpp(&self) -> Bpp {
        self.bpp
    }

    /// Returns an iterator over the raw pixels in this image.
    pub fn pixels(&self) -> RawPixels<'_> {
        RawPixels::new(self)
    }

    /// Returns the TGA header.
    ///
    /// The returned object is a direct representation of the header contained
    /// in the TGA file. Most of the information contained in the header is also
    /// available using other methods, which are the preferred way of accessing
    /// them.
    ///
    /// # Performance
    ///
    /// To save memory the header is parsed every time this method is called.
    pub fn header(&self) -> TgaHeader {
        // unwrap can't fail because the header was checked when self was created
        TgaHeader::parse(self.data).unwrap().1
    }

    /// Returns the developer directory.
    ///
    /// # Performance
    ///
    /// To save memory the footer is parsed every time this method is called.
    pub fn developer_directory(&self) -> Option<&'a [u8]> {
        TgaFooter::parse(self.data).and_then(|footer| footer.developer_directory(self.data))
    }

    /// Returns the extension area.
    ///
    /// # Performance
    ///
    /// To save memory the footer is parsed every time this method is called.
    pub fn extension_area(&self) -> Option<&'a [u8]> {
        TgaFooter::parse(self.data).and_then(|footer| footer.extension_area(self.data))
    }

    /// Returns the content of the image ID.
    ///
    /// If the TGA file doesn't contain an image ID `None` is returned.
    ///
    /// # Performance
    ///
    /// To save memory the header is parsed every time this method is called.
    pub fn image_id(&self) -> Option<&'a [u8]> {
        let (input, header) = TgaHeader::parse(self.data).ok()?;

        parse_image_id(input, &header)
            .ok()
            .map(|(_input, id)| id)
            .filter(|id| !id.is_empty())
    }
}

fn parse_image_id<'a>(input: &'a [u8], header: &TgaHeader) -> IResult<&'a [u8], &'a [u8]> {
    take(header.id_len)(input)
}
