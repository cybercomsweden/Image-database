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
        #[structopt(parse(from_str))]
        tag: String,
    },

    /// View and manage tags
    Tag(SubCmdTag),
}

#[derive(Debug, StructOpt)]
pub enum SubCmdTag {
    /// Add a new tag to the db, on the format: name type Option(parent)
    Add {
        /// Display name
        #[structopt(short = "n", long = "name")]
        name: String,

        /// Type of tag, may be "person", "event", "place" or "other"
        #[structopt(short = "t", long = "type")]
        tag_type: String,

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
