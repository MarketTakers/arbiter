use tonic_prost_build::configure;

static PROTOBUF_DIR: &str = "../../../protobufs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PROTOC").is_err() {
        println!("cargo:warning=PROTOC environment variable not set, using vendored protoc");
        let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
        unsafe { std::env::set_var("PROTOC", protoc) };
    }

    println!("cargo::rerun-if-changed={PROTOBUF_DIR}");

    configure()
        .message_attribute(".", "#[derive(::kameo::Reply)]")
        .compile_protos(
            &[
                format!("{}/arbiter.proto", PROTOBUF_DIR),
                format!("{}/user_agent.proto", PROTOBUF_DIR),
                format!("{}/client.proto", PROTOBUF_DIR),
                format!("{}/evm.proto", PROTOBUF_DIR),
            ],
            &[PROTOBUF_DIR.to_string()],
        )
        .unwrap();
    Ok(())
}
