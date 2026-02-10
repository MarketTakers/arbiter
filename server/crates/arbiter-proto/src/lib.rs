pub mod proto {
    tonic::include_proto!("arbiter");

    pub mod auth {
        tonic::include_proto!("arbiter.auth");
    }
}
