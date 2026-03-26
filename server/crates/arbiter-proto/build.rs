use std::path::PathBuf;
use tonic_prost_build::{Config, configure};

static PROTOBUF_DIR: &str = "../../../protobufs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed={PROTOBUF_DIR}");

    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    let protoc_include = protoc_bin_vendored::include_path()?;

    let mut config = Config::new();
    config.protoc_executable(protoc_path);

    let protos = [
        PathBuf::from(format!("{}/arbiter.proto", PROTOBUF_DIR)),
        PathBuf::from(format!("{}/user_agent.proto", PROTOBUF_DIR)),
        PathBuf::from(format!("{}/client.proto", PROTOBUF_DIR)),
        PathBuf::from(format!("{}/evm.proto", PROTOBUF_DIR)),
    ];

    let includes = [PathBuf::from(PROTOBUF_DIR), protoc_include];

    configure()
        .message_attribute(".", "#[derive(::kameo::Reply)]")
        .compile_with_config(config, &protos, &includes)?;
    Ok(())
}
