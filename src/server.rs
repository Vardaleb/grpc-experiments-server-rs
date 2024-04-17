use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub mod demo;
use demo::{
    demo_service_server::{DemoService, DemoServiceServer},
    DemoRequest, DemoResponse,
};

fn get_random_strings() -> Vec<String> {
    let mut rng = rand::thread_rng();
    let letters: Vec<String> = (0..5)
        .map(|_| {
            (0..10)
                .map(|_| rand::Rng::sample(&mut rng, rand::distributions::Alphanumeric) as char)
                .collect()
        })
        .collect();

    letters
}

#[derive(Debug, Default)]
pub struct DemoServiceImpl {}

#[tonic::async_trait]
impl DemoService for DemoServiceImpl {
    /// A unary gRPC function.
    /// 
    /// From the service defintion:
    /// ```proto
    /// service DemoService {
    ///   rpc Unary(DemoRequest) returns (DemoResponse);
    /// }
    /// ```
    async fn unary(&self, request: Request<DemoRequest>) -> Result<Response<DemoResponse>, Status> {
        println!("Got unary request {:?}", request);

        let query = request.into_inner().query;
        let reply = demo::DemoResponse {
            result: format!("Result for {}", query),
        };

        Ok(Response::new(reply))
    }

    /// A server streaming gRPC function.
    /// 
    /// From the service defintion:
    /// ```proto
    /// service DemoService {
    ///   rpc ServerStreaming(DemoRequest) returns (stream DemoResponse);
    /// }
    /// ```
    type ServerStreamingStream = ReceiverStream<Result<DemoResponse, Status>>;
    async fn server_streaming(
        &self,
        request: Request<DemoRequest>,
    ) -> Result<Response<Self::ServerStreamingStream>, Status> {
        println!("Got server streaming request {:?}", request);

        let (tx, rx) = tokio::sync::mpsc::channel(4);
        tokio::spawn(async move {
            for letter in get_random_strings() {
                tx.send(Ok(DemoResponse {
                    result: String::from(letter),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// A client streaming gRPC function.
    /// 
    /// From the service defintion:
    /// ```proto
    /// service DemoService {
    ///   rpc ClientStreaming(stream DemoRequest) returns (DemoResponse);
    /// }
    /// ```
    async fn client_streaming(
        &self,
        request: Request<Streaming<DemoRequest>>,
    ) -> Result<Response<DemoResponse>, Status> {
        println!("Got client streaming request {:?}", request);
        let mut result = String::new();
        let mut stream = request.into_inner();
        while let Some(element) = stream.next().await {
            if !result.is_empty() {
                result.push(',');
            }

            let element = element?;
            result.push_str(&format!("{}", element.query));
        }

        let reply = demo::DemoResponse { result };

        Ok(Response::new(reply))
    }

    /// A bidirectional streaming gRPC function.
    /// 
    /// From the service defintion:
    /// ```proto
    /// service DemoService {
    ///   rpc BidirectionalStreaming(stream DemoRequest) returns (stream DemoResponse);
    /// }
    /// ```
    type BidirectionalStreamingStream =
        Pin<Box<dyn Stream<Item = Result<DemoResponse, Status>> + Send + 'static>>;
    async fn bidirectional_streaming(
        &self,
        request: Request<Streaming<DemoRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingStream>, Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(request) = stream.next().await {
                let request = request?;
                let response = DemoResponse {
                    result: request.query.to_uppercase(),
                };

                yield response;
            }
        };

        Ok(Response::new(
            Box::pin(output) as Self::BidirectionalStreamingStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let demoserviceimpl = DemoServiceImpl::default();

    Server::builder()
        .add_service(DemoServiceServer::new(demoserviceimpl))
        .serve(addr)
        .await?;

    Ok(())
}
