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

use tokio::sync::Semaphore;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use warp::filters as wf;
use warp::reply::{with_status, Response};
use warp::{Filter, Reply, Rejection};

mod city;
use city::{Builder, Settings};

mod error;
use error::*;

const SOCKET: &str = "127.0.0.1:5000";
const SIMULTANEOUS_JOBS: usize = 3;

async fn map_rejections(rejection: Rejection) -> Result<Response, Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        println!("Error: {:?}", *error);
        Ok(error.into_response())
    } else {
        let body = ErrorJson {
            error: String::from(
                "{error: \"Something's wrong with our \
                 service, please report this to the developers \
                 with as much context as possible.\"}"
            ),
        };
        
        Ok(with_status(
            serde_json::to_string(&body).unwrap(),
            warp::http::status::StatusCode::INTERNAL_SERVER_ERROR,
        ).into_response())
    }
}

async fn generate_albedo(job_semaphore: Arc<Semaphore>, form: serde_json::Value) -> Result<Response, Rejection> {
    let permit = job_semaphore.try_acquire();

    if permit.is_err() {
        return Err(warp::reject::custom(Error::Overloaded));
    }
    
    if let Some(serde_json::Value::String(cityscript)) = form.get("cityscript") {
        let mut settings = Settings::default();

        settings.update(cityscript)
            .map_err(|err| Error::from(err))?;
        
        let config = settings.try_into()
            .map_err(|err| Error::from(err))?;

        let stream = Builder::new(config)
            .generate_roads().await
            .map_err(|err| Error::from(err))?
            .generate_buildings().await
            .map_err(|err| Error::from(err))?
            .build()
            .into_jpeg()
            .into_inner()
            .map_err(|_| Error::Server)?;

        Ok(Response::new(stream.into()))
    } else {
        // The form did not contain a cityscript in the right place
        Err(Error::Submission.into())
    }
}

fn log_requests(info: warp::log::Info) {
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
}

#[tokio::main]
async fn main() {
    let job_semaphore = Arc::new(Semaphore::new(SIMULTANEOUS_JOBS));

    // Set up a logging filter that logs all requests in detail
    let logger = warp::log::custom(log_requests);

    // Serve the page at the root path '/'
    let home = warp::path::end().and(wf::fs::file("static/main.html"));

    // Serve the static content at (and in) /static
    let static_filter = warp::path("static").and(wf::fs::dir("static"))
        .or(warp::path("docs").and(wf::fs::dir("static/docs")));

    // Serve the static content at (and in) /vendor
    let vendor_filter = warp::path("vendor").and(wf::fs::dir("vendor"));

    // Generate a 2048x2048 jpeg every time someone sends a request to the
    // /generate endpoint, unless the server is overloaded.
    let generate_filter = warp::path!("generate")
        .and(wf::body::content_length_limit(4096))
        .map(move || Arc::clone(&job_semaphore))
        .and(wf::body::json())
        .and_then(generate_albedo)
        .recover(map_rejections);

    // Serve all of our filters
    let static_filters = static_filter.or(vendor_filter);
    let all_filters = generate_filter.or(home).or(static_filters).with(logger);

    warp::serve(all_filters)
        .run(SocketAddr::from_str(SOCKET).unwrap())
        .await;
}
