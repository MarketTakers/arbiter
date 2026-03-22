use std::path::PathBuf;
use tonic_prost_build::configure;

static PROTOBUF_DIR: &str = "../../../protobufs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let protobuf_dir = manifest_dir.join(PROTOBUF_DIR);
    let protoc_include = protoc_bin_vendored::include_path()?;
    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;

    unsafe {
        std::env::set_var("PROTOC", &protoc_path);
        std::env::set_var("PROTOC_INCLUDE", &protoc_include);
    }

    println!("cargo::rerun-if-changed={}", protobuf_dir.display());

    configure()
        .message_attribute(".", "#[derive(::kameo::Reply)]")
        .compile_well_known_types(true)
        .compile_protos(
            &[
                protobuf_dir.join("arbiter.proto"),
                protobuf_dir.join("user_agent.proto"),
                protobuf_dir.join("client.proto"),
                protobuf_dir.join("evm.proto"),
            ],
            &[protobuf_dir],
        )?;
    Ok(())
}
