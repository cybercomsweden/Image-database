/*
Image database, allows the user to host a database themselves,
with the possibilities to tag and search after images.
Copyright (C) 2020 Cybercom group AB, Sweden
By Christoffer Dahl, Johanna Hultberg, Andreas Runfalk and Margareta Vi

Image database is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
*/
use anyhow::{anyhow, Context, Result};
use exif::Reader;
use image::{load_from_memory, DynamicImage, GenericImageView, ImageBuffer};
use std::convert::TryInto;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

use crate::face_detection::{calc_midpoint, face_detection, largest_bbox, Bbox};
use crate::metadata::{Metadata, Rotate, TypeSpecific};

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
    Jpeg,
    Png,
    Cr2,
    Nef,
    Dng,
}

impl FileType {
    pub fn media_type(&self) -> MediaType {
        match self {
            FileType::Mp4 | FileType::Mov => MediaType::Video,
            FileType::Jpeg | FileType::Png => MediaType::Image,
            FileType::Cr2 | FileType::Nef | FileType::Dng => MediaType::RawImage,
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

pub fn find_orientation(reader: &Reader) -> Option<Rotate> {
    let exif_orientation = &reader.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;

    match exif_orientation.value.get_uint(0)? {
        1 => Some(Rotate::Zero),
        3 => Some(Rotate::Cw180),
        6 => Some(Rotate::Cw90),
        8 => Some(Rotate::Ccw90),
        _ => None,
    }
}

fn get_video_snapshot<P: AsRef<Path>>(orig_path: P) -> Result<DynamicImage> {
    let metadata = Metadata::from_file(&orig_path)?;

    let video_metadata = match metadata.type_specific {
        TypeSpecific::Video(metadata) => metadata,
        TypeSpecific::Image(_) => return Err(anyhow!("Given file is an image")),
    };

    let skip_to = video_metadata.duration / 2.0;

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
        "jpg" | "jpeg" => Some(FileType::Jpeg),
        "png" => Some(FileType::Png),
        "cr2" => Some(FileType::Cr2),
        "nef" => Some(FileType::Nef),
        "mov" => Some(FileType::Mov),
        "mp4" => Some(FileType::Mp4),
        "dng" => Some(FileType::Dng),
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

pub fn copy_and_create_thumbnail<P: AsRef<Path>>(path: P) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let file_type = file_type_from_path(path.as_ref()).ok_or(anyhow!("Unknown file type"))?;
    let (img, rotation) = match file_type.media_type() {
        MediaType::Image => (
            image::open(path.as_ref()).context("failed to open image")?,
            {
                let file = fs::File::open(&path).unwrap();
                exif::Reader::new(&mut std::io::BufReader::new(&file))
                    .map(|x| find_orientation(&x).unwrap_or(Rotate::Zero))
                    .unwrap_or(Rotate::Zero)
            },
        ),
        MediaType::RawImage => (
            open_raw_image(path.as_ref()).context("failed to open raw image")?,
            Rotate::Zero,
        ),
        MediaType::Video => (get_video_snapshot(path.as_ref())?, Rotate::Zero),
    };

    let file_name = path.as_ref().file_stem().unwrap();
    let img = rotate_image(&img, rotation)?;

    fs::create_dir_all("dest")?;
    let dest_path = Path::new("dest");
    let copied_orig = dest_path.join(path.as_ref().file_name().unwrap());
    fs::copy(&path, &copied_orig)?;

    let preview_path = add_suffix(&dest_path.join(file_name), "_preview", ".jpg")?;
    create_preview(&img, &preview_path)?;

    let thumbnail = create_thumbnail_image(img)?;
    let thumbnail_path = add_suffix(&dest_path.join(file_name), "_thumbnail", ".jpg")?;
    thumbnail.save(&thumbnail_path)?;

    Ok((copied_orig, thumbnail_path, preview_path))
}

fn create_thumbnail_image(mut img: image::DynamicImage) -> Result<DynamicImage> {
    // Detect faces to determine where to optimally crop the image
    let detection = face_detection(&img);
    let thumbnail: image::DynamicImage;
    match detection {
        Ok(faces) => {
            if faces.len() == 0 {
                thumbnail = img.resize_to_fill(300, 200, image::FilterType::CatmullRom);
            } else {
                // Look for the largest face in the image and try to preserve it in the thumbnail
                let largest_bbox = largest_bbox(faces);
                let (start_x, start_y, new_width, new_height) =
                    calc_new_measurements(&img, largest_bbox);
                let tmp = img.crop(start_x, start_y, new_width, new_height);
                thumbnail = tmp.resize_exact(300, 200, image::FilterType::CatmullRom);
            }
        }
        Err(_) => process::exit(1),
    }
    Ok(thumbnail)
}

fn create_preview<P: AsRef<Path>>(img: &image::DynamicImage, preview_path: &P) -> Result<()> {
    let (width, height) = img.dimensions();
    let mut new_width = 4096;
    let mut new_height = 2160;
    if width > new_width || height > new_height {
        if height > width {
            let aspect_ratio = width as f32 / height as f32;
            new_width = (2160 as f32 * aspect_ratio).ceil() as u32;
        } else {
            let aspect_ratio = height as f32 / width as f32;
            new_height = (4096 as f32 * aspect_ratio).ceil() as u32;
        }
        let preview = img.resize_exact(new_width, new_height, image::FilterType::CatmullRom);
        preview.save(&preview_path)?;
    } else {
        img.save(&preview_path)?;
    }
    Ok(())
}

fn rotate_image(img: &DynamicImage, rotation: Rotate) -> Result<DynamicImage> {
    let rotated = match rotation {
        Rotate::Zero => img.clone(),
        Rotate::Cw90 => img.rotate90(),
        Rotate::Cw180 => img.rotate180(),
        Rotate::Ccw90 => img.rotate270(),
    };
    Ok(rotated)
}

pub fn copy_and_create_thumbnail_bytes(
    file_name: &str,
    data: &Vec<u8>,
) -> Result<(PathBuf, PathBuf, PathBuf)> {
    let slice = data.as_slice();

    fs::create_dir_all("dest")?;
    let dest_path = Path::new("dest");
    let copied_orig = dest_path.join(file_name);
    let file_path_new = format!("./dest/{}", &file_name);
    let mut file = std::fs::File::create(file_path_new.as_str())?;
    file.write_all(slice)?;

    let file_type = file_type_from_path(file_name).ok_or(anyhow!("Unknown file type"))?;
    let (img, rotation) = match file_type.media_type() {
        MediaType::Image => (
            load_from_memory(slice)?,
            exif::Reader::new(&mut std::io::BufReader::new(slice))
                .map(|x| find_orientation(&x).unwrap_or(Rotate::Zero))
                .unwrap_or(Rotate::Zero),
        ),
        MediaType::RawImage => (
            open_raw_image(file_path_new).context("failed to open raw image")?,
            Rotate::Zero,
        ),
        MediaType::Video => (
            match get_video_snapshot(&file_path_new) {
                Ok(o) => o,
                Err(e) => {
                    std::fs::remove_file(file_path_new.as_str())?;
                    return Err(e.into());
                }
            },
            Rotate::Zero,
        ),
    };

    let img = rotate_image(&img, rotation)?;

    let preview_tmp = format!("{}{}{}", file_name, "_preview", ".jpg");
    let preview_path = dest_path.join(preview_tmp.as_str());

    create_preview(&img, &preview_path)?;

    let thumbnail = create_thumbnail_image(img)?;

    let thumbnail_tmp = format!("{}{}{}", file_name, "_thumbnail", ".jpg");
    let thumbnail_path = dest_path.join(thumbnail_tmp.as_str());
    thumbnail.save(&thumbnail_path)?;

    Ok((copied_orig, thumbnail_path, preview_path))
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
