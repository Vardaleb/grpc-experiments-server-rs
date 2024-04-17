use std::{io::stdin, process::exit};

use demo::{demo_service_client::DemoServiceClient, DemoRequest};
use tonic::{transport::Channel, Request};

pub mod demo;

async fn call_unary(
    client: &mut DemoServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Please enter a request (enter to continue): ");
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim().to_string();
                if input.len() == 0 {
                    break;
                }
                let request = Request::new(DemoRequest { query: input });
                let response = client.unary(request).await?;
                println!("Got unary result: {}", response.into_inner().result);
            }
            Err(_) => exit(-1),
        }
    }
    Ok(())
}

async fn call_server_streaming(
    client: &mut DemoServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(DemoRequest {
        query: String::from(""),
    });
    let mut response_stream = client.server_streaming(request).await?.into_inner();

    while let Some(response) = response_stream.message().await? {
        println!("Got server streaming result: {}", response.result);
    }
    Ok(())
}

async fn call_client_streaming(
    client: &mut DemoServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let requests = vec![
        DemoRequest {
            query: String::from("A"),
        },
        DemoRequest {
            query: String::from("B"),
        },
        DemoRequest {
            query: String::from("C"),
        },
    ];
    let request = Request::new(tokio_stream::iter(requests));
    match client.client_streaming(request).await {
        Ok(response) => println!(
            "Got client streaming response = {}",
            response.into_inner().result
        ),
        Err(e) => println!("There was an error: {:?}", e),
    }
    Ok(())
}

async fn call_bidirectional_streaming(
    client: &mut DemoServiceClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let requests = vec![
        DemoRequest {
            query: String::from("x"),
        },
        DemoRequest {
            query: String::from("y"),
        },
        DemoRequest {
            query: String::from("z"),
        },
    ];
    let request = Request::new(tokio_stream::iter(requests));

    let mut response_stream = client.bidirectional_streaming(request).await?.into_inner();

    while let Some(response) = response_stream.message().await? {
        println!("Got bidirectional streaming result: {}", response.result);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DemoServiceClient::connect("http://[::1]:50051").await?;

    call_unary(&mut client).await?;
    call_server_streaming(&mut client).await?;
    call_client_streaming(&mut client).await?;
    call_bidirectional_streaming(&mut client).await?;

    println!("Good bye!");
    Ok(())
}
