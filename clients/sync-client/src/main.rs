use chrono::Local;
use hyper::{Client, Method, http::request::Builder, Body};
use hyper::body::HttpBody;

fn main() {

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    let client = Client::new();
    let req_builder = Builder::new().method(Method::POST);
    let req = req_builder
        .uri("http://127.0.0.1:3000")
        .body(Body::empty()).unwrap();
    let mut response = rt.block_on(client.request(req)).unwrap();

    while let Some(data) = rt.block_on(response.data()) {
        // println!("aaaaaaaaaaaaaaaaaaaaaaa: {:?}", data);
        if let Ok(d) = data {
            println!("block {:?}: {:?}", Local::now(), d);
            // tx.send(string);
        } else {
            println!("data_stream next value {:?}", data);
        }
    }

}
