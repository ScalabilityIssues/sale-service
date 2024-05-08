pub mod flightmngr {
    tonic::include_proto!("flightmngr");
}
pub mod priceest {
    tonic::include_proto!("priceestimator");
}
pub mod salesvc {
    tonic::include_proto!("salesvc");
}
pub mod ticketsrvc {
    tonic::include_proto!("ticketsrvc");
}
pub mod google {
    pub mod r#type {
        tonic::include_proto!("google.r#type");
    }
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("proto_descriptor");
