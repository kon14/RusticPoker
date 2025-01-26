use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Error: {0}")]
    Internal(String),
    #[error("Not Found: {0}")]
    NotFound(Uuid),
    #[error("Precondition Failed: {0}")]
    PreconditionFailed(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Invalid Request: {0}")]
    InvalidRequest(String),
}

impl AppError {
    pub fn internal<S>(info: S) -> Self
    where
        S: ToString,
    {
        Self::Internal(info.to_string())
    }

    pub fn not_found(uuid: Uuid) -> Self {
        Self::NotFound(uuid)
    }

    pub fn precondition_failed<S>(info: S) -> Self
    where
        S: ToString,
    {
        Self::PreconditionFailed(info.to_string())
    }

    pub fn unauthorized<S>(info: S) -> Self
    where
        S: ToString,
    {
        Self::Unauthorized(info.to_string())
    }

    pub fn invalid_request<S>(info: S) -> Self
    where
        S: ToString,
    {
        Self::InvalidRequest(info.to_string())
    }
}

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Internal(err) => Status::internal(err),
            AppError::NotFound(err) => Status::not_found(err),
            AppError::PreconditionFailed(err) => Status::failed_precondition(err),
            AppError::Unauthorized(err) => Status::permission_denied(err),
            AppError::InvalidRequest(err) => Status::invalid_argument(err),
        }
    }
}
