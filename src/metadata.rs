use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use exif::{In, Reader, Tag, Value};
use fraction::prelude::Fraction;
use image::GenericImageView;
use serde_json::{json, Value as serdeValue};
use std::convert::TryInto;
use std::fs;
use std::process::Command;

use crate::coord::{DecDegrees, Location};
use crate::thumbnail::{file_type_from_path, find_orientation, FileType, MediaType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rotate {
    Zero,
    Cw90,
    Ccw90,
    Cw180,
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub width: u32,
    pub height: u32,
    pub date_time: Option<DateTime<Utc>>,
    pub gps_location: Option<Location>,
    pub rotation: Option<Rotate>,
    pub type_specific: TypeSpecific,
}

#[derive(Clone, Debug, Default)]
pub struct ImageMetadata {
    pub exposure_time: Option<Fraction>,
    pub aperture: Option<f32>,
    pub iso: Option<u32>,
    pub flash: Option<bool>,
}
#[derive(Clone, Debug, Default)]
pub struct VideoMetadata {
    pub duration: f32,
    pub framerate: Option<f32>,
}
#[derive(Clone, Debug)]
pub enum TypeSpecific {
    Image(ImageMetadata),
    Video(VideoMetadata),
}

impl Metadata {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Metadata> {
        let file_type = file_type_from_path(&path).ok_or(anyhow!("Unknown file type"))?;
        let metadata = if file_type == FileType::Jpeg {
            extract_metadata_image_jpg(&path)?
        } else if file_type.media_type() == MediaType::Video {
            extract_metadata_video(&path)?
        } else {
            let width = 0;
            let height = 0;
            let date_time = None;
            let gps_location = None;
            let rotation = None;
            let type_specific = TypeSpecific::Image(ImageMetadata::default());
            Metadata {
                width,
                height,
                date_time,
                gps_location,
                rotation,
                type_specific,
            }
        };

        Ok(metadata)
    }
}

fn json_as_u64(json: &serde_json::Map<String, serdeValue>, key: &str) -> Result<u64> {
    Ok(json.get(key).map(|x| x.as_u64()).flatten().ok_or(anyhow!(
        "Key {} does not exist, or it is not an integer",
        key
    ))?)
}

fn json_as_object<'a>(
    json: &'a serde_json::Map<String, serdeValue>,
    key: &str,
) -> Result<&'a serde_json::Map<String, serdeValue>> {
    Ok(json
        .get(key)
        .map(|x| x.as_object())
        .flatten()
        .ok_or(anyhow!(
            "Key {} does not exist, or it is not an Object",
            key
        ))?)
}

fn get_leaf_value<'a>(
    json: &'a serde_json::map::Map<std::string::String, serde_json::value::Value>,
    name: &'a str,
) -> Result<&'a serde_json::value::Value> {
    Ok(json
        .get(name)
        .ok_or(anyhow!("Missing {} or key does not exist", name))?)
}

fn map_rotation(rot: &str) -> Result<Rotate> {
    match rot {
        "90" => Ok(Rotate::Cw90),
        "180" => Ok(Rotate::Cw180),
        "270" => Ok(Rotate::Ccw90),
        _ => Ok(Rotate::Zero),
    }
}

// the gps format is +58.3938+015.5612/
fn gps_video(coord: &str) -> Result<Location> {
    let coord = coord.replace("/", "");
    let split: Vec<String> = coord.split("+").map(|s| s.to_string()).collect();
    let lat = split[1].parse::<f64>()?;
    let lon = split[2].parse::<f64>()?;
    let place = Location::reverse_geolocation(lat, lon);
    Ok(Location::new(lat, lon, place))
}

fn field_as_string(reader: &Reader, tag: Tag) -> Option<String> {
    Some(
        reader
            .get_field(tag, In::PRIMARY)?
            .value
            .display_as(tag)
            .to_string(),
    )
}

fn field_as_uint(reader: &Reader, tag: Tag) -> Option<u32> {
    reader.get_field(tag, In::PRIMARY)?.value.get_uint(0)
}

fn width_and_height<P: AsRef<std::path::Path>>(path: P, reader: &Reader) -> Result<(u32, u32)> {
    let mut width = field_as_uint(&reader, Tag::ImageLength);
    let mut height = field_as_uint(&reader, Tag::ImageWidth);

    if width.is_none() || height.is_none() {
        width = field_as_uint(&reader, Tag::PixelXDimension);
        height = field_as_uint(&reader, Tag::PixelYDimension);
    }

    if width.is_none() || height.is_none() {
        Ok(image::open(path.as_ref())?.dimensions())
    } else {
        Ok((width.unwrap(), height.unwrap()))
    }
}

fn exposure_time(reader: &Reader) -> Option<Fraction> {
    let field = &reader.get_field(Tag::ExposureTime, In::PRIMARY)?.value;
    if let Value::Rational(exp_times) = field {
        // Note: this will panic if no exposure times present
        Some(Fraction::new(exp_times[0].num, exp_times[0].denom))
    } else {
        None
    }
}

fn aperture(reader: &Reader) -> Option<f32> {
    let field = &reader.get_field(Tag::ApertureValue, In::PRIMARY)?.value;
    let aperture = if let Value::Rational(ap) = field {
        // Note: this will panic if no aperture val present
        Some((ap[0].num as f32 / (2.0 * ap[0].denom as f32)).exp2())
    } else {
        None
    };

    if aperture.is_none() {
        let field = &reader.get_field(Tag::FNumber, In::PRIMARY)?.value;
        if let Value::Rational(f_val) = field {
            // Note: this will panic if no fnumber val present
            Some(f_val[0].num as f32 / f_val[0].denom as f32)
        } else {
            None
        }
    } else {
        aperture
    }
}

fn flash(reader: &Reader) -> Option<bool> {
    let flash = field_as_uint(&reader, Tag::Flash)?;
    let flash_fired = vec![
        0x1, 0x5, 0x7, 0x9, 0xd, 0xf, 0x19, 0x1d, 0x1f, 0x41, 0x45, 0x47, 0x49, 0x4d, 0x4f, 0x59,
        0x5d, 0x5f,
    ];
    if flash_fired.iter().any(|&f| f == flash) {
        Some(true)
    } else {
        Some(false)
    }
}

fn gps_image(reader: &Reader) -> Option<Location> {
    let lat_field = &reader.get_field(Tag::GPSLatitude, In::PRIMARY)?.value;
    let lon_field = &reader.get_field(Tag::GPSLongitude, In::PRIMARY)?.value;
    if let (Value::Rational(lat_dms), Value::Rational(lon_dms)) = (lat_field, lon_field) {
        let lat_is_positive = reader
            .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
            .map(|v| v.display_value().to_string() == "N")
            .unwrap_or(true);
        let lon_is_positive = reader
            .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
            .map(|v| v.display_value().to_string() == "E")
            .unwrap_or(true);
        Some(Location::from_dec_degrees(
            &DecDegrees::new(
                lat_dms[0].to_f64(),
                lat_dms[1].to_f64(),
                lat_dms[2].to_f64(),
                lat_is_positive,
            ),
            &DecDegrees::new(
                lon_dms[0].to_f64(),
                lon_dms[1].to_f64(),
                lon_dms[2].to_f64(),
                lon_is_positive,
            ),
        ))
    } else {
        None
    }
}

pub fn extract_metadata_image_jpg<P: AsRef<std::path::Path>>(path: P) -> Result<Metadata> {
    let file = fs::File::open(path.as_ref())?;
    let reader = Reader::new(&mut std::io::BufReader::new(&file))?;

    let date_time = field_as_string(&reader, Tag::DateTime);
    let date_time = date_time
        .and_then(|date_time| NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S").ok());
    let date_time = if let Some(v) = date_time {
        Some(DateTime::<Utc>::from_utc(v, Utc))
    } else {
        None
    };

    let rotation = find_orientation(&reader);
    let (width, height) = width_and_height(&path, &reader)?;
    let gps_location = gps_image(&reader);

    let mut image_metadata = ImageMetadata::default();
    image_metadata.exposure_time = exposure_time(&reader);
    image_metadata.aperture = aperture(&reader);
    image_metadata.iso = field_as_uint(&reader, Tag::PhotographicSensitivity);
    image_metadata.flash = flash(&reader);
    let type_specific = TypeSpecific::Image(image_metadata);

    Ok(Metadata {
        width,
        height,
        date_time,
        gps_location,
        rotation,
        type_specific,
    })
}

pub fn extract_metadata_video<P: AsRef<std::path::Path>>(path: P) -> Result<Metadata> {
    let file_name = path.as_ref().as_os_str();
    let proc = Command::new("ffprobe")
        .args(&["-v", "quiet"])
        .args(&["-print_format", "json"])
        .args(&["-show_format"])
        .args(&["-show_streams"])
        .arg(file_name)
        .output()?;

    let json_output: serdeValue = serde_json::from_str(std::str::from_utf8(&proc.stdout)?)?;
    let raw_metadata = json_output
        .as_object()
        .ok_or(anyhow!("Not a JSON object"))?;

    let format = json_as_object(&raw_metadata, "format")?;
    let duration = get_leaf_value(format, "duration")?
        .as_str()
        .ok_or(anyhow!("Duration is not string"))?
        .parse::<f32>()?;

    let tags = json_as_object(&format, "tags")?;

    let date_time = tags
        .get("creation_time")
        .and_then(|d| d.as_str())
        .and_then(|r| DateTime::parse_from_rfc3339(r).ok());

    let date_time = if let Some(v) = date_time {
        let timestamp = v.timestamp();
        let naive = NaiveDateTime::from_timestamp(timestamp, 0);
        Some(DateTime::<Utc>::from_utc(naive, Utc))
    } else {
        None
    };

    let framerate = tags
        .get("com.android.capture.fps")
        .and_then(|l| l.as_str())
        .and_then(|r| r.parse::<f32>().ok());

    let gps_string = if let Some(gps_string) = tags.get("location") {
        Some(gps_string)
    } else if let Some(gps_string) = tags.get("com.apple.quicktime.location.ISO6709") {
        Some(gps_string)
    } else {
        None
    };
    let gps_location = gps_string
        .and_then(|l| l.as_str())
        .and_then(|r| gps_video(r).ok());

    let mut video_metadata = VideoMetadata::default();
    video_metadata.duration = duration;
    video_metadata.framerate = framerate;

    let streams = get_leaf_value(&raw_metadata, "streams")?
        .as_array()
        .ok_or(anyhow!("Not an array"))?;
    for stream in streams {
        let stream = stream.as_object().ok_or(anyhow!("Not a JSON object"))?;
        if stream.get("codec_type") != Some(&json!("video")) {
            continue;
        }
        let width = json_as_u64(&stream, "width")?.try_into()?;
        let height = json_as_u64(&stream, "height")?.try_into()?;
        let rotation = json_as_object(&stream, "tags")?
            .get("rotate")
            .and_then(|rot| rot.as_str())
            .and_then(|r| map_rotation(r).ok());
        let type_specific = TypeSpecific::Video(video_metadata);

        return Ok(Metadata {
            width,
            height,
            date_time,
            gps_location,
            rotation,
            type_specific,
        });
    }

    Err(anyhow!("Unable to detect video stream")).into()
}
