use anyhow::anyhow;
use std::convert::TryInto;
use std::fmt;

use crate::error::Result;

fn to_dec_degrees(value: f64) -> (isize, isize, isize) {
    let d = value.trunc();
    let m = (60_f64 * (value - d)).abs().trunc();
    let s = 3600_f64 * (value - d).abs() - 60_f64 * m;
    return (d as isize, m as isize, s as isize);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn from_dec_degrees(
        lat_d: f64,
        lat_m: f64,
        lat_s: f64,
        lon_d: f64,
        lon_m: f64,
        lon_s: f64,
    ) -> Self {
        let lat = lat_d + lat_m / 60.0 + lat_s / 3600.0;
        let lon = lon_d + lon_m / 60.0 + lon_s / 3600.0;
        Self::new(lat, lon)
    }

    pub fn from_postgis_ewkb(raw: &[u8]) -> Result<Self> {
        // See https://github.com/postgis/postgis/blob/master/doc/ZMSgeoms.txt for information on
        // format structure
        if raw.len() != 25 {
            return Err(anyhow!("Unexpected length").into());
        }

        if raw[0] != 1 {
            return Err(anyhow!("Unexpected byte-order").into());
        }

        // We only support Points with SRID as of now (magic constant)
        if u32::from_le_bytes(raw[1..5].try_into()?) != 0x20000001 {
            return Err(anyhow!("Unexpected geometry type, must be Point with SRID").into());
        }

        // Validate SRID
        if u32::from_le_bytes(raw[5..9].try_into()?) != 4326 {
            return Err(anyhow!("Unexpected SRID").into());
        }

        let longitude = f64::from_le_bytes(raw[9..17].try_into()?);
        let latitude = f64::from_le_bytes(raw[17..25].try_into()?);

        Ok(Location {
            latitude,
            longitude,
        })
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Convert to S and W when negative
        let (lat_d, lat_m, lat_s) = to_dec_degrees(self.latitude);
        let (lon_d, lon_m, lon_s) = to_dec_degrees(self.longitude);
        write!(
            f,
            "{}° {}′ {}″ N, {}° {}′ {}″ E",
            lat_d, lat_m, lat_s, lon_d, lon_m, lon_s
        )
    }
}
