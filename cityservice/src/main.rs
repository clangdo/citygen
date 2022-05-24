//! Welcome to the citygen codebase. This project is about generating
//! abstract top-down city art. The service is implemented as a web
//! service using [warp].
//!
//! Very limited testing can be done in the standar manner with `cargo
//! test`. Note that any of these tests failing invalidates all
//! other test results.
//!
//! To make a request of the web service you can make a POST request
//! at the /generate endpoint of the webserver. This will generate and
//! return an image. To avoid resource hogging, the code limits the
//! number of concurrent jobs to 3, this allows for easy DOS on the
//! open internet, so please only run this project only on a trusted
//! network.
//!
//! For now [warp] listens on port 5000. Use this port to send
//! requests.
//!
//! The project also hosts a small web interface available at the root
//! endpoint. The interface allows you to input a configuration
//! "script" in "cityscript". You can "compile" the script using the
//! service and it will generate a city based on the input values. The
//! web interface will also display the result on the right hand side.
//!
//! This is really just a set of hierarchical key-value
//! pairs that dictates the behavior of the city generator. For more
//! information refer to the cityscript docs (coming soon).

use chrono::offset;

use tokio::sync::{Semaphore, SemaphorePermit};

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use warp::filters as wf;
use warp::reply::{with_status, Response};
use warp::{Filter, Reply, Rejection};

mod city;
use city::{Builder, Settings};

const SOCKET: &str = "127.0.0.1:5000";
const SIMULTANEOUS_JOBS: usize = 3;

#[derive(Debug)]
enum Error {
    Overloaded,
    Script,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

impl warp::reject::Reject for Error {}

impl Reply for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Overloaded => {
                with_status(
                    Response::new("Bad form values".into()),
                    warp::http::status::StatusCode::BAD_REQUEST,
                ).into_response()
            },
            Error::Script => {
                with_status(
                    "Bad script values or syntax error",
                    warp::http::status::StatusCode::BAD_REQUEST,
                ).into_response()
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let job_semaphore = Arc::new(Semaphore::new(SIMULTANEOUS_JOBS));

    // Set up a logging filter that logs all requests in detail
    let logger = warp::log::custom(|info| {
        let client_addr = match info.remote_addr() {
            Some(socket) => format!("{}", socket),
            None => String::from("unknown"),
        };

        let host_name = info.host().unwrap_or("unknown");

        let time = offset::Local::now();
        println!(
            "[{}][{}] from [{}] to [{}{}] took [{}ms]",
            time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            info.method(),
            client_addr,
            host_name,
            info.path(),
            info.elapsed().as_millis(),
        );
    });

    // Serve the page at the root path '/'
    let home = warp::path::end().and(wf::fs::file("static/main.html"));

    // Serve the static content at (and in) /static
    let static_filter = warp::path("static").and(wf::fs::dir("static"));

    // Serve the static content at (and in) /vendor
    let vendor_filter = warp::path("vendor").and(wf::fs::dir("vendor"));

    // Generate a 2048x2048 jpeg every time someone sends a request to the
    // /generate endpoint.
    let generate_filter = warp::path!("generate")
        .and(wf::body::content_length_limit(4096))
        .and(wf::body::json())
        .and_then(|form: serde_json::Value| async move {
            if let Some(serde_json::Value::String(cityscript)) = form.get("cityscript") {
                let mut settings = Settings::new();
                //settings.update(cityscript);
                let stream = Builder::new(settings)
                    .generate_roads().await
                    .unwrap()
                    .build()
                    .into_jpeg(2048, 2048)
                    .into_inner()
                    .expect("Could not unwrap image");

                Ok(Response::new(stream.into()))
            } else {
                // The form did contain a cityscript in the right place
                Err(warp::reject::custom(Error::Script))
            }
        });
        /*
        .recover(|rejection: Rejection| async {
            "Error"
        });*/

    // Serve all of our filters
    let static_filters = static_filter.or(vendor_filter);
    let all_filters = generate_filter.or(home).or(static_filters).with(logger);

    warp::serve(all_filters)
        .run(SocketAddr::from_str(SOCKET).unwrap())
        .await;
}
