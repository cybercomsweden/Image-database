# Image database
Personal digital photography organisation program that you host yourself.
With Rust as backend and React as frontend.
Ever wanted a way to manage all the photographies or videos that you have taken but do not want to upload it to the cloud?
with Image database you will be able to store them in one place. You will also be able to tag and if there are metadata about
where the images is taken you will also be able to show that exact locaction in a preview map.
You will also be able to get an overall world map that tells you where you have been, based on your photographs.

## Development environment setup
The following instructions are tested on:

* Fedora Workstation 31
* Ubuntu 18.04 LTS in Windows Subsystem for Linux (WSL)

First install Rust 1.40 or later using [Rustup](https://rustup.rs/).

The following command will automatically install all other requirements.

```bash
sudo ./env_setup.sh
```

## Frontend
```bash
npm install  # Only necessary when package.json has changed or when cloning the repository
npm run watch
```

Currently you can find the frontend at localhost:5000

## Backend

To setup the backend not via the env_setup.sh script:

```bash
service postgresql start
createdb nameOfDatabase
cargo run init-db
```

To import files via the terminal:

```bash
cargo run import path/to/your/images
```

To start the database

```bash
cargo run
```

## Usage

At localhost:5000 there are currently 4 different tabs, 'Media', 'Tags', 'Map', 'Upload' and a search bar.

Under tab 'Media' you will see thumbnails of all the images/video that you currently have uploaded to the database.
'Tags' tab is used for an overview of all the tags that currently exists in the database and to add new tags.
'Map' tab gives you an overview of where in the world that you have images.
'Upload' tab is used for drag and drop images to upload to the database.

Current file format that we know are supported are the following:
* Mp4
* Mov
* Jpeg
* Ong
* Cr2
* Nef
* Dng
