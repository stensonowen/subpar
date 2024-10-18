use protoc_rust as protoc;

const PROTO_FILES: &[&str] = &[
    "protos/gtfs-realtime.proto",
    // "protos/nyct-subway.proto",
];

fn main() {
    for watch in PROTO_FILES {
        println!("cargo:rerun-if-changed={}", watch);
    }
    protoc::Codegen::new()
        .out_dir("src/proto/")
        .inputs(PROTO_FILES)
        .include("protos/")
        .run()
        .expect("protoc failed");
}
