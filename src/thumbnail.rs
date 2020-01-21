use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer};
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    Image,
    RawImage,
    Video,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Rotate {
    Keep,
    Cw90,
    Ccw90,
    Cw180,
}

fn divide(x: u32, y: u32) -> u32 {
    let z = x as f32 / y as f32;
    z.ceil() as u32
}

fn ratio(x1: u32, y1: u32, x2: u32, y2: u32) -> u32 {
    let z1 = x1 as f32 / y1 as f32;
    let z2 = x2 as f32 / y2 as f32;
    (z1 / z2) as u32
}

fn add_suffix<T: AsRef<str>, U: AsRef<str>>(
    img_path: &std::path::Path,
    suffix: T,
    extension: U,
) -> Result<std::path::PathBuf> {
    let file_name = img_path
        .file_stem()
        .ok_or(anyhow!("Unable to determine file base name"))?;
    let mut os_string = std::ffi::OsString::from(file_name);
    os_string.push(suffix.as_ref());
    os_string.push(extension.as_ref());
    let mut dest_path = img_path.to_path_buf();
    dest_path.set_file_name(&os_string);
    Ok(dest_path)
}

fn create_thumbnail(img: &image::DynamicImage, x_size: u32, y_size: u32) -> image::DynamicImage {
    let (x, y) = img.dimensions();

    let (new_x, new_y, y_corner, x_corner) = if ratio(x, y, 3, 2) > 0 {
        let new_x = divide(x * y_size, y);
        let x_corner = divide(new_x, 2) - divide(x_size, 2);
        (new_x, y_size, 0, x_corner)
    } else {
        let new_y = divide(y * x_size, x);
        let y_corner = divide(new_y, 2) - divide(y_size, 2);
        (x_size, new_y, y_corner, 0)
    };

    let mut resized = img.resize(new_x, new_y, image::FilterType::Gaussian);
    resized.crop(x_corner, y_corner, x_size, y_size)
}

fn find_orientation<P: AsRef<std::path::Path>>(path: P) -> Option<Rotate> {
    let file = std::fs::File::open(path).unwrap();
    let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();

    let exif_orientation = reader.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;

    match exif_orientation.value.get_uint(0)? {
        6 => Some(Rotate::Cw90),
        1 => Some(Rotate::Keep),
        _ => None,
    }
}

pub fn media_type_from_path<P: AsRef<Path>>(path: P) -> Option<MediaType> {
    let ext = path.as_ref().extension()?.to_str()?;
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" => Some(MediaType::Image),
        "cr2" | "nef" => Some(MediaType::RawImage),
        "mov" | "mp4" => Some(MediaType::Video),
        _ => None,
    }
}

pub fn open_raw_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
    let srgb_img = imagepipe::simple_decode_8bit(path, 0, 0).map_err(|e| anyhow!("{}", e))?;
    let buf = ImageBuffer::from_raw(
        srgb_img.width.try_into()?,
        srgb_img.height.try_into()?,
        srgb_img.data,
    )
    .ok_or(anyhow!("Failed to convert raw to image"))?;
    Ok(DynamicImage::ImageRgb8(buf))
}

pub fn copy_and_create_thumbnail<P: AsRef<Path>>(path: P) -> Result<(PathBuf, PathBuf)> {
    let (img, rotation) =
        match media_type_from_path(path.as_ref()).ok_or(anyhow!("Unknown file type"))? {
            MediaType::Image => (
                image::open(path.as_ref())?,
                find_orientation(path.as_ref()).unwrap_or(Rotate::Keep),
            ),
            MediaType::RawImage => (open_raw_image(path.as_ref())?, Rotate::Keep),
            MediaType::Video => {
                return Err(anyhow!("No handler for video")).into();
            }
        };

    let file_name = path.as_ref().file_stem().unwrap();

    // Create and save the corresponding thumbnail
    let dest_path = std::path::Path::new("dest");
    fs::create_dir_all("dest")?;
    let img = match rotation {
        Rotate::Keep => img,
        Rotate::Cw90 => img.rotate90(),
        Rotate::Cw180 => img.rotate180(),
        Rotate::Ccw90 => img.rotate270(),
    };
    let thumbnail = create_thumbnail(&img, 300, 200);
    let thumbnail_path = add_suffix(&dest_path.join(file_name), "_resized", ".jpg")?;
    thumbnail.save(&thumbnail_path)?;

    // Copy the original image to the destination folder
    let img_path = dest_path.join(path.as_ref().file_name().unwrap());
    fs::copy(&path, &img_path)?;
    Ok((img_path, thumbnail_path))
}
