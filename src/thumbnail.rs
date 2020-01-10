/// fn _example_usage()-> Result<()> {
///     // Use the open function to load an image from a Path.
///     // ```open``` returns a `DynamicImage` on success.
///     let img_path = std::path::Path::new("/home/johanna/Bilder/person.jpeg");
///     let img = image::open(img_path)?;
///
///     let thumbnail = create_thumbnail(&img, 300, 200);
///
///     let dest_path = add_suffix(img_path, "_resized", ".jpg")?;
///
///     // Write the resized and cropped image to a .jpg file
///     thumbnail.save(&dest_path)?;
///     Ok(())
/// }


use image::GenericImageView;
use anyhow::{anyhow, Result};

fn divide(x: u32, y: u32) -> u32 {
    let z = x as f32 / y as f32;
    z.ceil() as u32
}

fn ratio(x1: u32, y1: u32, x2: u32, y2: u32) -> u32 {
    let z1 = x1 as f32 / y1 as f32;
    let z2 = x2 as f32 / y2 as f32;
    (z1 / z2) as u32
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

pub fn create_thumbnail(img: &image::DynamicImage, x_size: u32, y_size: u32) -> image::DynamicImage {
    let (x, y) = img.dimensions();

    let (new_x, new_y, y_corner, x_corner) = if ratio(x, y, 3, 2) > 0 {
        let new_x = divide(x * y_size, y);
        let x_corner = divide(new_x, 2) - divide(x_size, 2);
        (new_x, y_size, 0, x_corner)
    } else {
        let new_y = divide(y * x_size, x);
        let y_corner = divide(new_y, 2) - divide(y_size, 2);
        (x_size, new_y, y_corner, 0)
    };

    let mut resized = img.resize(new_x, new_y, image::FilterType::Gaussian);
    resized.crop(x_corner, y_corner, x_size, y_size)
}

