use anyhow::{anyhow, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer};
use serde_json::{json, Value};
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

struct VideoMetadata {
    width: u32,
    height: u32,
    duration: f32,
    rotation: Rotate,
}

fn json_as_u64(json: &serde_json::Map<String, Value>, key: &str) -> Result<u64> {
    Ok(json.get(key).map(|x| x.as_u64()).flatten().ok_or(anyhow!(
        "Key {} does not exist, or it is not an integer",
        key
    ))?)
}

fn json_as_object<'a>(
    json: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>> {
    Ok(json
        .get(key)
        .map(|x| x.as_object())
        .flatten()
        .ok_or(anyhow!(
            "Key {} does not exist, or it is not an Object",
            key
        ))?)
}

impl VideoMetadata {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file_name = path.as_ref().as_os_str();
        let proc = Command::new("ffprobe")
            .args(&["-v", "quiet"])
            .args(&["-print_format", "json"])
            .args(&["-show_format"])
            .args(&["-show_streams"])
            .arg(file_name)
            .output()?;

        let json_output: Value = serde_json::from_str(std::str::from_utf8(&proc.stdout)?)?;
        let raw_metadata = json_output
            .as_object()
            .ok_or(anyhow!("Not a JSON object"))?;

        let format = json_as_object(&raw_metadata, "format")?;
        let duration = format
            .get("duration")
            .ok_or(anyhow!("Missing duration"))?
            .as_str()
            .ok_or(anyhow!("Duration not string"))?
            .parse::<f32>()?;

        let streams = raw_metadata
            .get("streams")
            .ok_or(anyhow!("No streams detected"))?
            .as_array()
            .ok_or(anyhow!("Not an array"))?;
        for stream in streams {
            let stream = stream.as_object().ok_or(anyhow!("Not a JSON object"))?;
            if stream.get("codec_type") != Some(&json!("video")) {
                continue;
            }
            let width = json_as_u64(&stream, "width")?.try_into()?;
            let height = json_as_u64(&stream, "height")?.try_into()?;

            let rotate = json_as_object(&stream, "tags")?.get("rotate");
            let rotation = match rotate.map(|r| r.as_str()).flatten() {
                Some("90") => Rotate::Cw90,
                Some("180") => Rotate::Cw180,
                Some("270") => Rotate::Ccw90,
                _ => Rotate::Keep,
            };
            return Ok(Self {
                duration,
                width,
                height,
                rotation,
            });
        }

        Err(anyhow!("Unable to detect video stream")).into()
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
    let file = std::fs::File::open(path).unwrap();
    let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();

    let exif_orientation = reader.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;

    match exif_orientation.value.get_uint(0)? {
        6 => Some(Rotate::Cw90),
        1 => Some(Rotate::Keep),
        _ => None,
    }
}

fn get_video_snapshot<P: AsRef<Path>>(orig_path: P) -> Result<DynamicImage> {
    let metadata = VideoMetadata::from_file(orig_path.as_ref())?;

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
        .output()?;

    let (height, width) = if metadata.rotation == Rotate::Keep || metadata.rotation == Rotate::Cw180
    {
        (metadata.height, metadata.width)
    } else {
        (metadata.width, metadata.height)
    };
    let buf = ImageBuffer::from_raw(width, height, proc.stdout)
        .ok_or(anyhow!("Failed to convert raw to image"))?;
    Ok(DynamicImage::ImageRgb8(buf))
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
            MediaType::Video => (get_video_snapshot(path.as_ref())?, Rotate::Keep),
        };

    let file_name = path.as_ref().file_stem().unwrap();

    let img = match rotation {
        Rotate::Keep => img,
        Rotate::Cw90 => img.rotate90(),
        Rotate::Cw180 => img.rotate180(),
        Rotate::Ccw90 => img.rotate270(),
    };

    fs::create_dir_all("dest")?;
    let dest_path = Path::new("dest");
    let copied_orig = dest_path.join(path.as_ref().file_name().unwrap());
    fs::copy(&path, &copied_orig)?;

    let seam_carv = seam_carving(img);
    let thumbnail = seam_carv.resize_exact(300, 200, image::FilterType::Gaussian);
    let thumbnail_path = add_suffix(&dest_path.join(file_name), "_resized", ".jpg")?;
    thumbnail.save(&thumbnail_path)?;

    Ok((copied_orig, thumbnail_path))
}

pub fn seam_carving(img: image::DynamicImage) -> image::DynamicImage {
    let (width, height) = img.dimensions();
    let aspect_ratio = width as f32 / height as f32;
    if aspect_ratio as f32 == 1.5 {
        // already 3:2 format
        return img;
    } else if aspect_ratio as f32 > 1.5 {
        let new_width = (height as f32 * 1.5).ceil() as u32;
        return DynamicImage::ImageRgba8(seamcarving::resize(&img, new_width, height));
    } else {
        let new_height = (width as f32 / 1.5).ceil() as u32;
        return DynamicImage::ImageRgba8(seamcarving::resize(&img, width, new_height));
    }
}
