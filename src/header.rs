use nom::{
    combinator::{map, map_opt, map_res},
    number::complete::{le_u16, le_u8},
    IResult,
};

use crate::parse_error::ParseError;

/// Bits per pixel.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum Bpp {
    /// 8 bits per pixel.
    Bits8,
    /// 16 bits per pixel.
    Bits16,
    /// 24 bits per pixel.
    Bits24,
    /// 32 bits per pixel.
    Bits32,
}

impl Bpp {
    fn new(value: u8) -> Option<Self> {
        Some(match value {
            8 => Self::Bits8,
            16 => Self::Bits16,
            24 => Self::Bits24,
            32 => Self::Bits32,
            _ => return None,
        })
    }

    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map_opt(le_u8, Bpp::new)(input)
    }

    fn parse_opt(input: &[u8]) -> IResult<&[u8], Option<Self>> {
        map(le_u8, Bpp::new)(input)
    }

    /// Returns the number of bits.
    pub fn bits(self) -> u8 {
        match self {
            Self::Bits8 => 8,
            Self::Bits16 => 16,
            Self::Bits24 => 24,
            Self::Bits32 => 32,
        }
    }

    /// Returns the number of bytes needed to store values with this bit depth.
    pub fn bytes(self) -> u8 {
        match self {
            Self::Bits8 => 1,
            Self::Bits16 => 2,
            Self::Bits24 => 3,
            Self::Bits32 => 4,
        }
    }
}

/// Image data compression.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Compression {
    /// Uncompressed image data.
    Uncompressed,
    /// Run-length encoded image data.
    Rle,
}

/// Image data type.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DataType {
    /// No image data.
    NoData,
    /// Color mapped.
    ColorMapped,
    /// True color.
    TrueColor,
    /// Black and white or grayscale.
    BlackAndWhite,
}

fn parse_image_type(image_type: u8) -> Result<(DataType, Compression), ParseError> {
    if image_type & !0b1011 != 0 || image_type == 8 {
        return Err(ParseError::UnsupportedImageType(image_type));
    }

    let data_type = match image_type & 0x3 {
        1 => DataType::ColorMapped,
        2 => DataType::TrueColor,
        3 => DataType::BlackAndWhite,
        _ => DataType::NoData,
    };

    let compression = if image_type & 0x8 != 0 {
        Compression::Rle
    } else {
        Compression::Uncompressed
    };

    Ok((data_type, compression))
}

/// Image origin
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ImageOrigin {
    /// Bottom left
    BottomLeft,
    /// Bottom right
    BottomRight,
    /// Top left
    TopLeft,
    /// Top right
    TopRight,
}

impl ImageOrigin {
    fn from_image_descriptor(value: u8) -> Self {
        match (value & 0x30) >> 4 {
            0 => Self::BottomLeft,
            1 => Self::BottomRight,
            2 => Self::TopLeft,
            _ => Self::TopRight,
        }
    }

    pub(crate) fn is_bottom(self) -> bool {
        match self {
            Self::BottomLeft | Self::BottomRight => true,
            _ => false,
        }
    }
}

/// TGA header.
///
/// See <https://www.fileformat.info/format/tga/egff.htm> for a detailed description of the fields.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct TgaHeader {
    /// Image ID field length
    pub id_len: u8,

    /// Whether a color map is included in the image data
    pub has_color_map: bool,

    /// Data type.
    pub data_type: DataType,

    /// Compression.
    pub compression: Compression,

    /// Color map origin
    pub color_map_start: u16,

    /// Length of color map
    pub color_map_len: u16,

    /// Number of bits in each color palette entry
    pub color_map_depth: Option<Bpp>,

    /// Image origin (X)
    pub x_origin: u16,

    /// Image origin (Y)
    pub y_origin: u16,

    /// Image width in pixels
    pub width: u16,

    /// Image height in pixels
    pub height: u16,

    /// Pixel bit depth
    pub pixel_depth: Bpp,

    /// Image origin
    pub image_origin: ImageOrigin,

    /// Alpha channel depth
    pub alpha_channel_depth: u8,
}

impl TgaHeader {
    pub(crate) fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, id_len) = le_u8(input)?;
        let (input, has_color_map) = has_color_map(input)?;
        let (input, (data_type, compression)) = map_res(le_u8, parse_image_type)(input)?;
        let (input, color_map_start) = le_u16(input)?;
        let (input, color_map_len) = le_u16(input)?;
        let (input, color_map_depth) = Bpp::parse_opt(input)?;
        let (input, x_origin) = le_u16(input)?;
        let (input, y_origin) = le_u16(input)?;
        let (input, width) = le_u16(input)?;
        let (input, height) = le_u16(input)?;
        let (input, pixel_depth) = Bpp::parse(input)?;

        let (input, image_descriptor) = le_u8(input)?;
        let image_origin = ImageOrigin::from_image_descriptor(image_descriptor);
        let alpha_channel_depth = image_descriptor & 0xF;

        Ok((
            input,
            TgaHeader {
                id_len,
                has_color_map,
                data_type,
                compression,
                color_map_start,
                color_map_len,
                color_map_depth,
                x_origin,
                y_origin,
                width,
                height,
                pixel_depth,
                image_origin,
                alpha_channel_depth,
            },
        ))
    }
}

fn has_color_map(input: &[u8]) -> IResult<&[u8], bool> {
    map_res(le_u8, |b| match b {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ParseError::ColorMap),
    })(input)
}
