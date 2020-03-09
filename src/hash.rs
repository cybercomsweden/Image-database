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
use async_std::fs::File as AsyncFile;
use async_std::io::{Read as AsyncRead, ReadExt};
use sha3::digest::Digest;
use std::convert::TryInto;
use std::fmt;
use std::path::Path;

use crate::error::Result;

type HashType = [u8; 32];

#[derive(Clone, PartialEq, Eq)]
pub struct Sha3 {
    hash: HashType,
}

impl Sha3 {
    pub fn as_ref(&self) -> &[u8] {
        &self.hash
    }

    pub fn try_from_slice(value: &[u8]) -> Result<Self> {
        Ok(Self {
            hash: value.try_into()?,
        })
    }

    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = AsyncFile::open(path.as_ref().to_owned()).await?;
        Ok(Self::from_reader(file).await?)
    }

    pub async fn from_reader<R: AsyncRead + Unpin>(mut reader: R) -> Result<Self> {
        let mut buf = [0u8; 4096]; // Use 4096 as the buffer size
        let mut hasher = sha3::Sha3_256::new();
        loop {
            let buf_len = reader.read(&mut buf).await?;
            if buf_len == 0 {
                break;
            }
            hasher.input(&buf[..buf_len]);
        }

        Ok(hasher.result().into())
    }
}

impl<T: Into<HashType>> From<T> for Sha3 {
    fn from(value: T) -> Self {
        Self { hash: value.into() }
    }
}

impl fmt::Debug for Sha3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sha3(\"{}\")", self)
    }
}

impl fmt::Display for Sha3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.as_ref().iter() {
            write!(f, "{:02x}", byte)?
        }
        Ok(())
    }
}
