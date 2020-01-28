use std::error::Error;

use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions, SessionRunArgs, Tensor};

use image::GenericImageView;

#[derive(Copy, Clone, Debug)]
pub struct Bbox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub prob: f32,
}

// x,y for the length of bounding box
fn calc_length(bbox: Bbox) -> (f32, f32) {
    ((bbox.x2 - bbox.x1), (bbox.y2 - bbox.y1))
}

// x_mid, y_mid
fn calc_midpoint(bbox: Bbox) -> (u32, u32) {
    let (width_len, height_len) = calc_length(bbox);
    (
        (bbox.x1 + width_len / 2.0) as u32,
        (bbox.y1 + height_len / 2.0) as u32,
    )
}

pub fn face_detection(input_image: &image::DynamicImage) -> Result<Vec<Bbox>, Box<dyn Error>> {
    let model = include_bytes!("mtcnn.pb");

    let mut graph = Graph::new();
    graph.import_graph_def(&*model, &ImportGraphDefOptions::new())?;

    let mut flattened: Vec<f32> = Vec::new();

    // model uses BGR instead of RGB, converting.
    for (_x, _y, rgb) in input_image.pixels() {
        flattened.push(rgb[2] as f32);
        flattened.push(rgb[1] as f32);
        flattened.push(rgb[0] as f32);
    }

    let input = Tensor::new(&[input_image.height() as u64, input_image.width() as u64, 3])
        .with_values(&flattened)?;

    let mut session = Session::new(&SessionOptions::new(), &graph)?;
    let min_size = Tensor::new(&[]).with_values(&[150f32])?;
    let thresholds = Tensor::new(&[3]).with_values(&[0.6f32, 0.7f32, 0.7f32])?;
    let factor = Tensor::new(&[]).with_values(&[0.709f32])?;

    let mut args = SessionRunArgs::new();

    //load param for model
    args.add_feed(&graph.operation_by_name_required("min_size")?, 0, &min_size);
    args.add_feed(
        &graph.operation_by_name_required("thresholds")?,
        0,
        &thresholds,
    );
    args.add_feed(&graph.operation_by_name_required("factor")?, 0, &factor);

    // load input image
    args.add_feed(&graph.operation_by_name_required("input")?, 0, &input);

    // output args
    let bbox = args.request_fetch(&graph.operation_by_name_required("box")?, 0);
    let prob = args.request_fetch(&graph.operation_by_name_required("prob")?, 0);

    session.run(&mut args)?;

    let bbox_res: Tensor<f32> = args.fetch(bbox)?; //number of faces x 4
    let prob_res: Tensor<f32> = args.fetch(prob)?; // num faces

    let bboxes: Vec<_> = bbox_res
        .chunks_exact(4)
        .zip(prob_res.iter())
        .map(|(bbox, &prob)| Bbox {
            y1: bbox[0],
            x1: bbox[1],
            y2: bbox[2],
            x2: bbox[3],
            prob,
        })
        .collect();
    Ok(bboxes)
}
