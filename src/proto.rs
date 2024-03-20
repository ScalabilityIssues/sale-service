pub mod flightmngr {
    tonic::include_proto!("flightmngr");
}
pub mod priceest {
    tonic::include_proto!("priceestimator");
}
pub mod salesvc {
    tonic::include_proto!("salesvc");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("proto_descriptor");
