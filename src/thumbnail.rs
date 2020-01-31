use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer};
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

use crate::face_detection::{calc_midpoint, face_detection, largest_bbox, Bbox};
use crate::metadata::{Rotate, VideoMetadata};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    Image,
    RawImage,
    Video,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    Mp4,
    Mov,
    Jpg,
    Jpeg,
    Png,
    Cr2,
    Nef,
}

impl FileType {
    pub fn media_type(&self) -> MediaType {
        match self {
            FileType::Mp4 | FileType::Mov => MediaType::Video,
            FileType::Jpg | FileType::Jpeg | FileType::Png => MediaType::Image,
            FileType::Cr2 | FileType::Nef => MediaType::RawImage,
        }
    }
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

fn find_orientation<P: AsRef<std::path::Path>>(path: P) -> Option<Rotate> {
    let file = fs::File::open(path).unwrap();
    let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();

    let exif_orientation = reader.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;

    match exif_orientation.value.get_uint(0)? {
        1 => Some(Rotate::Zero),
        3 => Some(Rotate::Cw180),
        6 => Some(Rotate::Cw90),
        8 => Some(Rotate::Ccw90),
        _ => None,
    }
}

fn get_video_snapshot<P: AsRef<Path>>(orig_path: P) -> Result<DynamicImage> {
    let metadata =
        VideoMetadata::from_file(orig_path.as_ref()).context("failed to get metadata, video")?;

    let skip_to = metadata.duration / 2.0;

    let orig_path = orig_path
        .as_ref()
        .to_str()
        .ok_or(anyhow!("Could not convert to str"))?;

    let proc = Command::new("ffmpeg")
        .args(&["-loglevel", "-8"]) // silent, does not log anything
        .args(&["-ss", &skip_to.to_string()]) // skip into half the video
        .args(&["-i", orig_path]) // the path to the video
        .args(&["-frames:v", "1"]) // only save one frame
        .args(&["-f", "image2pipe"])
        .args(&["-pix_fmt", "rgb24"]) // 24-bit RGB format for pixels
        .args(&["-vcodec", "rawvideo"])
        .arg("-") // Output to stdout
        .output()
        .context("failed to extract thumbnail")?;

    let (height, width) = if metadata.rotation == Some(Rotate::Zero)
        || metadata.rotation == Some(Rotate::Cw180)
        || metadata.rotation == None
    {
        (metadata.height, metadata.width)
    } else {
        (metadata.width, metadata.height)
    };
    let buf = ImageBuffer::from_raw(width, height, proc.stdout)
        .ok_or(anyhow!("Failed to convert video frame to image"))?;
    Ok(DynamicImage::ImageRgb8(buf))
}

pub fn file_type_from_path<P: AsRef<Path>>(path: P) -> Option<FileType> {
    let ext = path.as_ref().extension()?.to_str()?;
    match ext.to_ascii_lowercase().as_str() {
        "jpg" => Some(FileType::Jpg),
        "jpeg" => Some(FileType::Jpeg),
        "png" => Some(FileType::Png),
        "cr2" => Some(FileType::Cr2),
        "nef" => Some(FileType::Nef),
        "mov" => Some(FileType::Mov),
        "mp4" => Some(FileType::Mp4),
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
    let (img, rotation) = match file_type_from_path(path.as_ref())
        .ok_or(anyhow!("Unknown file type"))?
        .media_type()
    {
        MediaType::Image => (
            image::open(path.as_ref()).context("failed to open image")?,
            find_orientation(path.as_ref()).unwrap_or(Rotate::Zero),
        ),
        MediaType::RawImage => (
            open_raw_image(path.as_ref()).context("failed to open raw image")?,
            Rotate::Zero,
        ),
        MediaType::Video => (get_video_snapshot(path.as_ref())?, Rotate::Zero),
    };

    let file_name = path.as_ref().file_stem().unwrap();

    let mut img = match rotation {
        Rotate::Zero => img,
        Rotate::Cw90 => img.rotate90(),
        Rotate::Cw180 => img.rotate180(),
        Rotate::Ccw90 => img.rotate270(),
    };

    fs::create_dir_all("dest")?;
    let dest_path = Path::new("dest");
    let copied_orig = dest_path.join(path.as_ref().file_name().unwrap());
    fs::copy(&path, &copied_orig)?;

    let detection = face_detection(&img);
    let tmp: image::DynamicImage;
    match detection {
        Ok(faces) => {
            // only carve if we do not have any visible faces in the image
            if faces.len() == 0 {
                tmp = seam_carving(&img);
            } else {
                let largest_bbox = largest_bbox(faces);
                let (start_x, start_y, new_width, new_height) =
                    calc_new_measurements(&img, largest_bbox);
                tmp = img.crop(start_x, start_y, new_width, new_height);
            }
        }
        Err(_) => process::exit(1),
    }

    let thumbnail = tmp.resize_exact(300, 200, image::FilterType::CatmullRom);
    let thumbnail_path = add_suffix(&dest_path.join(file_name), "_thumbnail", ".jpg")?;
    thumbnail.save(&thumbnail_path)?;

    Ok((copied_orig, thumbnail_path))
}

fn seam_carving(img: &image::DynamicImage) -> image::DynamicImage {
    let (width, height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;
    if aspect_ratio as f32 == 1.5 {
        // already 3:2 format
        return img.clone();
    } else if aspect_ratio as f32 > 1.5 {
        let new_width = (height as f32 * 1.5).ceil() as u32;
        return DynamicImage::ImageRgba8(seamcarving::resize(img, new_width, height));
    } else {
        let new_height = (width as f32 / 1.5).ceil() as u32;
        return DynamicImage::ImageRgba8(seamcarving::resize(img, width, new_height));
    }
}

// return a tuple with (start_x, start_y, width, height)
fn calc_new_measurements(img: &image::DynamicImage, bbox: Bbox) -> (u32, u32, u32, u32) {
    let (width, height) = img.dimensions();
    let (x_mid, y_mid) = calc_midpoint(bbox);
    let aspect_ratio = width as f32 / height as f32;
    if aspect_ratio as f32 == 1.5 {
        // already 3:2 format
        return (0, 0, width, height);
    } else if aspect_ratio as f32 > 1.5 {
        let new_width = (height as f32 * 1.5).ceil() as u32;
        let diff = x_mid as i32 - new_width as i32 / 2;
        if diff >= 0 {
            return (diff as u32, 0, new_width, height);
        } else {
            return (0, 0, new_width, height);
        }
    } else {
        let new_height = (width as f32 / 1.5).ceil() as u32;
        let diff = y_mid as i32 - new_height as i32 / 2;
        if diff >= 0 {
            return (0, diff as u32, width, new_height);
        } else {
            return (0, 0, width, new_height);
        }
    }
}
