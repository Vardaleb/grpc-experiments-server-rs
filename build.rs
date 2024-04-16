fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["grpc-experiments-messages/proto/service.proto"],
        &["grpc-experiments-messages/proto"],
    )?;
    Ok(())
}
