use spacetimedb::Identity;
use std::error::Error as StdError;
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("E400: {0}")]
    BadRequest(String),
    #[error("E401: {0}")]
    Unauthorized(String),
    #[error("E403: {0}")]
    Forbidden(String),
    #[error("E404: {0}")]
    NotFound(String),
    #[error("E409: {0}")]
    Conflict(String),
    #[error("E418: {0}")]
    Validation(String),
    #[error("E429: {0}")]
    RateLimited(String),
    #[error("E500: {0}")]
    Internal(String),
}

impl ServiceError {
    pub fn unauthorized(sender: Identity, reason: impl Into<String>) -> Self {
        ServiceError::Unauthorized(format!("Unauthorized (sender={sender}): {}", reason.into()))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        ServiceError::Internal(message.into())
    }
}

pub trait ErrorMapper {
    fn map_bad_request_error(self) -> ServiceError;
    fn map_unauthorized_error(self) -> ServiceError;
    fn map_forbidden_error(self) -> ServiceError;
    fn map_not_found_error(self) -> ServiceError;
    fn map_conflict_error(self) -> ServiceError;
    fn map_validation_error(self) -> ServiceError;
    fn map_rate_limited_error(self) -> ServiceError;
    fn map_internal_error(self) -> ServiceError;
}

impl<E> ErrorMapper for E
where
    E: StdError + Send + Sync + 'static,
{
    fn map_bad_request_error(self) -> ServiceError {
        ServiceError::BadRequest(self.to_string())
    }

    fn map_unauthorized_error(self) -> ServiceError {
        ServiceError::Unauthorized(self.to_string())
    }

    fn map_forbidden_error(self) -> ServiceError {
        ServiceError::Forbidden(self.to_string())
    }

    fn map_not_found_error(self) -> ServiceError {
        ServiceError::NotFound(self.to_string())
    }

    fn map_conflict_error(self) -> ServiceError {
        ServiceError::Conflict(self.to_string())
    }

    fn map_validation_error(self) -> ServiceError {
        ServiceError::Validation(self.to_string())
    }

    fn map_rate_limited_error(self) -> ServiceError {
        ServiceError::RateLimited(self.to_string())
    }

    fn map_internal_error(self) -> ServiceError {
        ServiceError::Internal(self.to_string())
    }
}

pub trait ResultExt<T, E: StdError + Send + Sync + 'static> {
    fn map_bad_request(self) -> ServiceResult<T>;
    fn map_unauthorized(self) -> ServiceResult<T>;
    fn map_forbidden(self) -> ServiceResult<T>;
    fn map_not_found(self) -> ServiceResult<T>;
    fn map_conflict(self) -> ServiceResult<T>;
    fn map_validation(self) -> ServiceResult<T>;
    fn map_rate_limited(self) -> ServiceResult<T>;
    fn map_internal(self) -> ServiceResult<T>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn map_bad_request(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_bad_request_error())
    }

    fn map_unauthorized(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_unauthorized_error())
    }

    fn map_forbidden(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_forbidden_error())
    }

    fn map_not_found(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_not_found_error())
    }

    fn map_conflict(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_conflict_error())
    }

    fn map_validation(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_validation_error())
    }

    fn map_rate_limited(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_rate_limited_error())
    }

    fn map_internal(self) -> ServiceResult<T> {
        self.map_err(|e| e.map_internal_error())
    }
}
