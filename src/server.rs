use std::{pin::Pin, sync::Arc};

use demoservice::{
    demo_service_server::{DemoService, DemoServiceServer},
    DemoRequest, DemoResponse,
};
use tokio::sync::Mutex;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub mod demoservice {
    tonic::include_proto!("demo");
}

#[derive(Debug, Default)]
pub struct DemoServiceImpl {
    queries: Arc<Mutex<Vec<String>>>,
}

impl DemoServiceImpl {
    pub fn new() -> DemoServiceImpl {
        DemoServiceImpl {
            queries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[allow(unused)]
#[tonic::async_trait]
impl DemoService for DemoServiceImpl {
    // rpc Unary(DemoRequest) returns (DemoResponse);
    async fn unary(&self, request: Request<DemoRequest>) -> Result<Response<DemoResponse>, Status> {
        println!("Got request {:?}", request);

        let query = request.into_inner().query;
        // remember query
        let mut lock = self.queries.lock().await;
        lock.push(query.clone());

        let reply = demoservice::DemoResponse {
            result: format!("Result for {}", query),
        };

        println!("Queries so far: {:?}", lock);
        Ok(Response::new(reply))
    }

    // rpc ServerStreaming(DemoRequest) returns (stream DemoResponse);
    type ServerStreamingStream = ReceiverStream<Result<DemoResponse, Status>>;
    async fn server_streaming(
        &self,
        request: Request<DemoRequest>,
    ) -> Result<Response<Self::ServerStreamingStream>, Status> {
        println!("Got request {:?}", request);

        let (mut tx, rx) = tokio::sync::mpsc::channel(4);

        let queries = self.queries.lock().await.clone();

        tokio::spawn(async move {
            for query in queries.iter() {
                tx.send(Ok(DemoResponse {
                    result: query.clone(),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    // rpc ClientStreaming(stream DemoRequest) returns (DemoResponse);
    async fn client_streaming(
        &self,
        request: Request<Streaming<DemoRequest>>,
    ) -> Result<Response<DemoResponse>, Status> {
        unimplemented!()
    }

    // rpc BidirectionalStreaming(stream DemoRequest) returns (stream DemoResponse);
    type BidirectionalStreamingStream =
        Pin<Box<dyn Stream<Item = Result<DemoResponse, Status>> + Send + 'static>>;
    async fn bidirectional_streaming(
        &self,
        request: Request<Streaming<DemoRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingStream>, Status> {
        unimplemented!()
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
