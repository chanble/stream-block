use std::time::SystemTime;

use futures_util::FutureExt;
use hyper::{Client, Method, http::request::Builder, Body};
use hyper::body::HttpBody;
use chrono::prelude::*;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let req_builder = Builder::new().method(Method::POST);
    let req = req_builder
        .uri("http://127.0.0.1:3000")
        .body(Body::empty()).unwrap();
    let mut response = client.request(req).await.unwrap();

    // 
    //  因为服务返回的是chunked的数据，本来想用读stream的方式，一块一块的返回数据
    // 百般尝试后， 发现这种直接读response.data也可以分块打印数据
    // 猜测应该有个超时时间，如果一段时间内不再返回数据，则先返回这些
    // response.data().into_stream();

    while let Some(data) = response.data().await {
        // println!("aaaaaaaaaaaaaaaaaaaaaaa: {:?}", data);
        if let Ok(d) = data {
            // let string = String::from_utf8(d.into_iter().collect::<Vec<u8>>()).unwrap_or_default();
            println!("{:?}: {:?}", Local::now(), d);
        } else {
            println!("data_stream next value {:?}", data);
        }
    }
}