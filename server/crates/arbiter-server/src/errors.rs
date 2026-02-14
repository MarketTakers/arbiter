use tonic::Status;
use tracing::error;

pub trait GrpcStatusExt<T> {
    fn to_status(self) -> Result<T, Status>;
}

impl<T> GrpcStatusExt<T> for Result<T, diesel::result::Error> {
    fn to_status(self) -> Result<T, Status> {
         self.map_err(|e| {
            error!(error = ?e, "Database error");
            Status::internal("Database error")
        })
    }
}

impl<T> GrpcStatusExt<T> for Result<T, crate::db::PoolError> {
    fn to_status(self) -> Result<T, Status> {
        self.map_err(|e| {
            error!(error = ?e, "Database pool error");
            Status::internal("Database pool error")
        })
    }
}