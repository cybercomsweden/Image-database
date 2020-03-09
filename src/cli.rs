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
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Start the application web service [default]
    Run,

    /// Import supported media recursively from the given folder
    Import {
        /// One or more files or directories to import
        #[structopt(parse(from_os_str), required = true)]
        paths: Vec<PathBuf>,
    },

    /// Initialize database
    InitDb,

    /// Show metadata for the provided media
    Metadata {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    Search {
        /// One or more tags to search for
        #[structopt(parse(from_str), required = true)]
        tags: Vec<String>,
    },

    /// View and manage tags
    Tag(SubCmdTag),

    /// Delete
    Delete {
        /// Id of media to delete
        #[structopt(short = "i", long = "id")]
        id: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum SubCmdTag {
    /// Add a new tag to the db, on the format: name type Option(parent)
    Add {
        /// Display name
        #[structopt(short = "n", long = "name")]
        name: String,

        /// ID of parent tag. If not provided the tag is considered a top level tag
        #[structopt(short = "p", long = "parent")]
        parent: Option<String>,
    },

    /// List all present tags and their relation
    List,

    /// Add tag to image
    Image {
        #[structopt(parse(from_os_str))]
        path: PathBuf,

        #[structopt(short = "t", long = "tag")]
        tag: String,
    },

    /// Add parent to existing tag
    AddParent {
        #[structopt(short = "t", long = "tag")]
        tag: String,

        #[structopt(short = "p", long = "parent")]
        parent: String,
    },

    /// Remove parent to existing tag
    RemoveParent {
        #[structopt(short = "t", long = "tag")]
        tag: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub cmd: Option<Cmd>,
}

impl Args {
    pub fn from_args() -> Self {
        // Expose from_args so we don't have to import StructOpt anywhere but here
        <Self as StructOpt>::from_args()
    }
}
