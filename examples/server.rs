use std::error::Error;

use bytes::Bytes;
use h2::server;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = env_logger::try_init();

    let mut listener = TcpListener::bind("127.0.0.1:5928").await?;

    println!("listening on {:?}", listener.local_addr());

    while let Ok((socket, _peer_addr)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle(socket).await {
                println!("  -> err={:?}", e);
            }
        });
    }

    println!("Server terminated");
    Ok(())
}

async fn handle(socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = server::Builder::new()
        .max_concurrent_streams(1)
        .handshake(socket)
        .await?;
    println!("H2 connection bound");

    while let Some(result) = connection.accept().await {
        let (request, respond) = result?;
        tokio::spawn(async move {
            if let Err(e) = handle_req(request, respond).await {
                println!("    -> err={:?}", e);
            }
        });
    }

    println!("Connection terminated");
    Ok(())
}

async fn handle_req(
    mut request: http::Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("GOT request: {:?}", request);
    let body = request.body_mut();
    while !body.is_end_stream() {
        println!("Waiting for body");
        if let Some(data) = body.data().await {
            println!("BODY data: {:?}", data?);
        } else {
            println!("BODY trailer: {:?}", body.trailers().await?);
        }
    }
    println!("GOT body: {:?}", body);
    let response = http::Response::new(());

    let mut send = respond.send_response(response, false)?;

    println!(">>>> sending response data");
    send.send_data(Bytes::from_static(b"hello world"), true)?;

    Ok(())
}
