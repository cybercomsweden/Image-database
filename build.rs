fn main() {
    prost_build::compile_protos(&["src/entity.proto"], &["src/"])
        .expect("Failed to generate rust protobuf files. Have you installed protobuf compiler?");
}
