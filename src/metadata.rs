use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use exif::{In, Reader, Tag, Value};
use fraction::prelude::Fraction;
use image::GenericImageView;
use serde_json::{json, Value as serdeValue};
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::coord::Location;

#[derive(Clone, Debug)]
pub struct Metadata {
    width: u32,
    height: u32,
    date_time: Option<NaiveDateTime>, // Should be DateTime, not naive
    exposure_time: Option<Fraction>,
    aperture: Option<f32>,
    iso: Option<u32>,
    flash: Option<bool>,
    gps_location: Option<Location>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rotate {
    Keep,
    Cw90,
    Ccw90,
    Cw180,
}

pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration: f32,
    pub rotation: Rotate,
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

impl VideoMetadata {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
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

fn field_as_string(reader: &Reader, tag: Tag) -> Option<String> {
    Some(
        reader
            .get_field(tag, In::PRIMARY)?
            .value
            .display_as(Tag::DateTime)
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

fn gps(reader: &Reader) -> Option<Location> {
    let lat_field = &reader.get_field(Tag::GPSLatitude, In::PRIMARY)?.value;
    let lon_field = &reader.get_field(Tag::GPSLongitude, In::PRIMARY)?.value;
    if let (Value::Rational(lat_dms), Value::Rational(lon_dms)) = (lat_field, lon_field) {
        Some(Location::from_dec_degrees(
            lat_dms[0].to_f64(),
            lat_dms[1].to_f64(),
            lat_dms[2].to_f64(),
            lon_dms[0].to_f64(),
            lon_dms[1].to_f64(),
            lon_dms[2].to_f64(),
        ))
    } else {
        None
    }
}

pub fn extract_metadata<P: AsRef<std::path::Path>>(path: P) -> Result<Metadata> {
    let file = fs::File::open(path.as_ref())?;
    let reader = Reader::new(&mut std::io::BufReader::new(&file))?;

    let date_time = field_as_string(&reader, Tag::DateTime);
    let date_time = date_time
        .and_then(|date_time| NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S").ok());

    let (width, height) = width_and_height(&path, &reader)?;
    let exposure_time = exposure_time(&reader);
    let aperture = aperture(&reader);
    let iso = field_as_uint(&reader, Tag::PhotographicSensitivity);
    let flash = flash(&reader);
    let gps_location = gps(&reader);

    Ok(Metadata {
        date_time,
        width,
        height,
        exposure_time,
        aperture,
        iso,
        flash,
        gps_location,
    })
}
