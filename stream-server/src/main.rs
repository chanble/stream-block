use std::convert::Infallible;
use std::error::Error;
use std::thread;
use std::time::Duration;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // pretty_env_logger::init();

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    //
    let chunked_body = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
        49, 50,
    ];
    let stream = tokio_stream::iter(chunked_body.into_iter().map(|v| {
        thread::sleep(Duration::from_secs(3));

        Result::<_, Infallible>::Ok(format!("{},", v))
    }));
    let body = Body::wrap_stream(stream);
    Ok(Response::new(body))
}
