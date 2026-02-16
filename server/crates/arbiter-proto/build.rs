static PROTOBUF_DIR: &str = "../../../protobufs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = vec![
        format!("{}/arbiter.proto", PROTOBUF_DIR),
        format!("{}/auth.proto", PROTOBUF_DIR),
    ];

    // Компилируем protobuf (tonic-prost-build автоматически использует prost_types для google.protobuf)
    tonic_prost_build::configure()
        .message_attribute(".", "#[derive(::kameo::Reply)]")
        .compile_protos(&proto_files, &[PROTOBUF_DIR.to_string()])?;

    Ok(())
}
