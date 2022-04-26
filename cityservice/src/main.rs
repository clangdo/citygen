use std::net::SocketAddr;
use std::str::FromStr;
use std::io::{Write, BufWriter};

use warp::reply::Response;
use warp::{Filter};
use warp::filters as wf;

const SOCKET: &str = "127.0.0.1:5000";

#[tokio::main]
async fn main() {
    // Serve the page at the root path '/'
    let home = warp::path::end()
        .and(wf::fs::file("static/main.html"));

    // Serve the static content at (and in) /static
    let static_filter = warp::path("static").and(wf::fs::dir("static"));

    // Generate a 2048x2048 jpeg every time someone sends a request to the
    // /generate endpoint.
    let generate_filter = warp::path!("generate")
        .map(|| {
            let image = image::DynamicImage::new_rgb8(2048, 2048);
            let jpeg = Vec::with_capacity(2050 * 2048);
            let mut buf_writer = BufWriter::new(jpeg);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut buf_writer);
            encoder.encode_image(&image).expect("Failed to encode image to memory");
            buf_writer.flush().expect("Failed to flush buffer");

            Response::new(buf_writer.into_inner().expect("Could not unwrap image").into())
        });

    // Serve all of our filters
    warp::serve(home.or(generate_filter).or(static_filter))
        .run(SocketAddr::from_str(SOCKET).unwrap()).await;
}
