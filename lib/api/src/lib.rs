pub const MESSAGES_FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("messages_descriptor");

tonic::include_proto!("flux.messages");

pub const STREAMS_FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("streams_descriptor");

tonic::include_proto!("flux.streams");

tonic::include_proto!("flux.notify");
