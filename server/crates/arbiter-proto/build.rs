use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure()
        .proto_path("../../../protobufs")
        .compile_protos(&["../../../protobufs/auth.proto"], &["../../../protobufs"])
        .unwrap();
    Ok(())
}
