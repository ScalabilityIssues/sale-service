use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("error contacting sub-service: {0}")]
    SubServiceError(#[from] tonic::Status),
}

impl From<ApplicationError> for tonic::Status {
    fn from(error: ApplicationError) -> tonic::Status {
        match error {
            _ => {
                tracing::error!(%error, "internal error");
                tonic::Status::internal("internal error")
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, ApplicationError>;
