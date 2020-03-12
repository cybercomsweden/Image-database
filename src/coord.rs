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
use anyhow::anyhow;
use rgeo::search;
use std::convert::TryInto;
use std::fmt;

use crate::error::Result;

pub struct DecDegrees {
    pub d: f64,
    pub m: f64,
    pub s: f64,
    pub is_positive: bool,
}

impl DecDegrees {
    pub fn new(d: f64, m: f64, s: f64, is_positive: bool) -> Self {
        Self {
            d,
            m,
            s,
            is_positive,
        }
    }

    pub fn from_scalar(value: f64) -> Self {
        let d = value.trunc();
        let m = (60_f64 * (value - d)).abs().trunc();
        let s = 3600_f64 * (value - d).abs() - 60_f64 * m;
        Self {
            d,
            m,
            s,
            is_positive: value >= 0.0,
        }
    }

    pub fn to_scalar(&self) -> f64 {
        let value = self.d + self.m / 60.0 + self.s / 3600.0;
        if self.is_positive {
            value
        } else {
            -value
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub place: String,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64, place: String) -> Self {
        Self {
            latitude,
            longitude,
            place,
        }
    }

    pub fn from_dec_degrees(lat: &DecDegrees, lon: &DecDegrees) -> Self {
        let lat = lat.to_scalar();
        let lon = lon.to_scalar();
        let place = Self::reverse_geolocation(lat, lon);
        Self::new(lat, lon, place)
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

        let place = Self::reverse_geolocation(latitude, longitude);
        Ok(Location {
            latitude,
            longitude,
            place,
        })
    }

    pub fn reverse_geolocation(lat: f64, lon: f64) -> String {
        let (_, record) = search(lat as f32, lon as f32).unwrap();
        format!("{}, {}", record.name, record.country)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lat = DecDegrees::from_scalar(self.latitude);
        let lon = DecDegrees::from_scalar(self.longitude);
        write!(
            f,
            "{}° {}′ {}″ {}, {}° {}′ {}″ {}",
            lat.d,
            lat.m,
            lat.s,
            if lat.is_positive { 'N' } else { 'S' },
            lon.d,
            lon.m,
            lon.s,
            if lat.is_positive { 'E' } else { 'W' },
        )
    }
}
