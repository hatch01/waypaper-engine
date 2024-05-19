use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use crate::file_reading_utils::{read_null_terminated_str, read_u32};

#[derive(Debug, Clone)]
pub enum TextureFormat {
    RGBA8888 = 0,
    DXT5 = 4,
    DXT3 = 6,
    DXT1 = 7,
    RG88 = 8,
    R8 = 9,
}

impl TryFrom<u32> for TextureFormat {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => TextureFormat::RGBA8888,
            4 => TextureFormat::DXT5,
            6 => TextureFormat::DXT3,
            7 => TextureFormat::DXT1,
            8 => TextureFormat::RG88,
            9 => TextureFormat::R8,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone)]
pub enum TextureFlags {
    None = 0,
    NoInterpolation = 1,
    ClampUVs = 2,
    IsGIF = 4,
}

impl TryFrom<u32> for TextureFlags {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => TextureFlags::None,
            1 => TextureFlags::NoInterpolation,
            2 => TextureFlags::ClampUVs,
            4 => TextureFlags::IsGIF,
            _ => return Err(()),
        })
    }
}

pub struct ContainerData {
    version: ContainerVersion,
    unknown_data: u32,
    freeimage_format: Option<u32>,
    mipmap_levels: u32,
}

pub struct MipmapEntry {
    width: u32,
    height: u32,
    is_compressed: bool,
    image_size_uncompressed: Option<u32>,
    image_size: u32,
    mipmap_pixels: Vec<u8>,
}

pub enum ContainerVersion {
    TEXB001,
    TEXB002,
    TEXB003,
}

impl TryFrom<u32> for ContainerVersion {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => ContainerVersion::TEXB001,
            2 => ContainerVersion::TEXB002,
            3 => ContainerVersion::TEXB003,
            _ => return Err(()),
        })
    }
}

pub struct Header {
    format: TextureFormat,
    flags: TextureFlags,
    texture_width: u32,
    texture_height: u32,
    image_width: u32,
    image_height: u32,
    unknown_funny_number: u32,
}

pub struct TexFile {
    header: Header,
    container_data: ContainerData,
    mipmap_entries: Vec<MipmapEntry>,
}

impl TexFile {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Unpacking Tex File !");

        let mut data: Cursor<Vec<u8>> = Cursor::new(fs::read(path)?);
        println!("Data Length : {}", data.get_ref().len());

        let header = read_header(&mut data);
        let container_data = read_container_data(&mut data);

        let mut mipmap_entries = vec![];
        for i in 0..container_data.mipmap_levels {
            println!("Reading Mipmap {i} :");
            mipmap_entries.push(read_mipmap(&mut data, &container_data.version));
        }

        Ok(Self {
            header,
            container_data,
            mipmap_entries,
        })
    }
}

fn read_header(data: &mut Cursor<Vec<u8>>) -> Header {
    let version = read_null_terminated_str(data);
    assert_eq!(version, "TEXV0005");
    let version2 = read_null_terminated_str(data);
    assert_eq!(version2, "TEXI0001");

    println!("{version} - {version2}");

    let format = TextureFormat::try_from(read_u32(data)).unwrap();
    let flags = TextureFlags::try_from(read_u32(data)).unwrap(); // TODO: Flags can probably be combined
    let texture_width = read_u32(data);
    let texture_height = read_u32(data);
    let image_width = read_u32(data);
    let image_height = read_u32(data);
    let unknown_funny_number = read_u32(data);

    println!("Texture info:");
    println!("\tFormat: {:?}", format);
    println!("\tFlags: {:?}", flags);
    println!("\tTexture Size: {texture_width}x{texture_height}");
    println!("\tImage Size: {image_width}x{image_height}");
    println!("\tUnknown funny number: {unknown_funny_number}");

    Header {
        format,
        flags,
        texture_width,
        texture_height,
        image_width,
        image_height,
        unknown_funny_number,
    }
}

fn read_container_data(data: &mut Cursor<Vec<u8>>) -> ContainerData {
    let container_version_str = read_null_terminated_str(data);
    println!("Container version: {container_version_str}");

    assert!(container_version_str == "TEXB0001" || container_version_str == "TEXB0002" || container_version_str == "TEXB0003");
    let version = ContainerVersion::try_from(container_version_str.chars().last().unwrap().to_digit(10).unwrap()).unwrap();

    let unknown_data = read_u32(data);
    let freeimage_format = match version {
        ContainerVersion::TEXB001 | ContainerVersion::TEXB002 => {
            None
        }
        ContainerVersion::TEXB003 => {
            Some(read_u32(data))
        }
    };
    let mipmap_levels = read_u32(data);

    println!("\tUnknown funny number 2: {unknown_data}");
    if let Some(format) = freeimage_format {
        println!("\tFreeimage Format: {format}");
    }
    println!("\tMipmap levels: {mipmap_levels}");

    ContainerData {
        version,
        freeimage_format,
        mipmap_levels,
        unknown_data,
    }
}

fn read_mipmap(cursor: &mut Cursor<Vec<u8>>, container_version: &ContainerVersion) -> MipmapEntry {
    let width = read_u32(cursor);
    let height = read_u32(cursor);

    let (is_compressed, image_size_uncompressed) = match container_version {
        ContainerVersion::TEXB001 => (false, None),
        ContainerVersion::TEXB002 | ContainerVersion::TEXB003 => {
            let compression_flag = read_u32(cursor);
            assert!(compression_flag == 0 || compression_flag == 1);
            let is_compressed = compression_flag != 0;

            let image_size_uncompressed = read_u32(cursor);

            (is_compressed, Some(image_size_uncompressed))
        }
    };

    let image_size = read_u32(cursor);

    println!("\tWidth: {width}");
    println!("\tHeight: {height}");
    println!("\tIs Compressed: {is_compressed}");

    if is_compressed {
        println!("\tImage Size Uncompressed: {}", image_size_uncompressed.unwrap());
    }

    println!("\tImage Size: {image_size}", );

    let mut bytes = vec![];
    cursor.take(image_size as u64).read_to_end(&mut bytes).unwrap();

    MipmapEntry {
        width,
        height,
        is_compressed,
        image_size_uncompressed,
        image_size,
        mipmap_pixels: vec![],
    }
}