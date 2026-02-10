use tonic_prost_build::configure;

static PROTOBUF_DIR: &str = "../../../protobufs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure()
        .compile_protos(
            &[
                format!("{}/arbiter.proto", PROTOBUF_DIR),
                format!("{}/auth.proto", PROTOBUF_DIR),
            ],
            &[PROTOBUF_DIR.to_string()],
        )
        .unwrap();
    Ok(())
}
