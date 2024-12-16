use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("messages_descriptor.bin"))
        .compile_protos(&["src/messages.proto"], &["src"])
        .unwrap();

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("streams_descriptor.bin"))
        .compile_protos(&["src/streams.proto"], &["src"])
        .unwrap();

    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["src/notify.proto"], &["src"])
        .unwrap();
}
