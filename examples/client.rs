use h2::client;
use http::{HeaderMap, Request};

use std::error::Error;

use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let tcp = TcpStream::connect("127.0.0.1:5928").await?;
    let (client, h2) = client::handshake(tcp).await?;

    println!("sending request");

    let request = Request::builder()
        .uri("https://http2.akamai.com/")
        .body(())
        .unwrap();
    let request2 = Request::builder()
        .uri("https://http2.akamai.com/")
        .body(())
        .unwrap();

    let mut trailers = HeaderMap::new();
    trailers.insert("zomg", "hello".parse().unwrap());

    let mut client = client.ready().await?;
    let (response, mut stream) = client.send_request(request, false)?;
    let mut client = client.ready().await?;
    let (response2, _) = client.send_request(request2, true)?;

    stream.send_data(bytes::Bytes::from(&b"hello"[..]), false)?;

    // send trailers
    stream.send_trailers(trailers).unwrap();

    // Spawn a task to run the conn...
    tokio::spawn(async move {
        if let Err(e) = h2.await {
            println!("GOT ERR={:?}", e);
        }
    });

    let (response, response2) = futures::join!(response, response2);
    let (response, response2) = (response?, response2?);
    println!("GOT RESPONSE: {:?}", response);
    println!("GOT RESPONSE 2: {:?}", response2);

    // Get the body
    let mut body = response.into_body();

    while let Some(chunk) = body.data().await {
        println!("GOT CHUNK = {:?}", chunk?);
    }

    if let Some(trailers) = body.trailers().await? {
        println!("GOT TRAILERS: {:?}", trailers);
    }

    Ok(())
}
