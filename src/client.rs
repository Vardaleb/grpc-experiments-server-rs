use std::{io::stdin, process::exit};

use demoservice::{demo_service_client::DemoServiceClient, DemoRequest};
use tonic::Request;

pub mod demoservice {
    tonic::include_proto!("demo");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DemoServiceClient::connect("http://[::1]:50051").await?;
    loop {
        println!("Please enter a request: ");
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim().to_string();
                if input.len() == 0 {
                    break;
                }
                let request = Request::new(DemoRequest { query: input });
                println!("About to send unary request {:?}", request);
                let response = client.unary(request).await?;
                println!("Got unary result: {:?}", response);
            }
            Err(_) => exit(-1),
        }
    }

    let request = Request::new(DemoRequest {
        query: String::from(""),
    });
    println!("About to send server streaming request {:?}", request);
    let mut response_stream = client.server_streaming(request).await?.into_inner();

    while let Some(response) = response_stream.message().await? {
        println!("Got server streaming result: {:?}", response);
    }

    println!("Good bye!");
    Ok(())
}
