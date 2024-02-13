use std::fs;
use std::path::Path;

const PROTO_WATCH: &str = "/proto";
const PROTO_PATH: &str = "proto/rustic_poker.proto";
const PROTO_GEN_PATH: &str = "src/proto";
const FILE_DESCRIPTOR_PATH: &str = "src/proto/rustic_poker_descriptor.bin";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prepare();
    build_protos();
    Ok(())
}

fn prepare() {
    let path = Path::new(PROTO_GEN_PATH);
    if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
    }
    fs::create_dir_all(path).unwrap();
}

fn build_protos() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(PROTO_GEN_PATH)
        .file_descriptor_set_path(FILE_DESCRIPTOR_PATH)
        .compile(&[PROTO_PATH], &[PROTO_WATCH])
        .unwrap();
}
