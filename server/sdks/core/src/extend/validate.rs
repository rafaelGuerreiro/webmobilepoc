use crate::{
    error::{ErrorMapper, ServiceError, ServiceResult},
    repository::character::{CharacterV1, services::CharacterReducerContext},
};
use spacetimedb::ReducerContext;
use thiserror::Error;

pub trait ReducerContextRequirements {
    fn validate_u8(&self, value: u8, name: impl Into<String>, min_value: u8, max_value: u8) -> ServiceResult<()> {
        self.validate_u64(value as u64, name, min_value as u64, max_value as u64)
    }

    fn validate_u16(&self, value: u16, name: impl Into<String>, min_value: u16, max_value: u16) -> ServiceResult<()> {
        self.validate_u64(value as u64, name, min_value as u64, max_value as u64)
    }

    fn validate_u32(&self, value: u32, name: impl Into<String>, min_value: u32, max_value: u32) -> ServiceResult<()> {
        self.validate_u64(value as u64, name, min_value as u64, max_value as u64)
    }

    fn validate_u64(&self, value: u64, name: impl Into<String>, min_value: u64, max_value: u64) -> ServiceResult<()> {
        validate_u64(value, name, min_value, max_value)
    }

    fn validate_str(&self, value: &str, name: impl Into<String>, min_len: u64, max_len: u64) -> ServiceResult<()> {
        validate_str(value, name, min_len, max_len)
    }

    fn require_internal_access(&self) -> ServiceResult<()>;

    fn require_online(&self) -> ServiceResult<CharacterV1>;
}

impl ReducerContextRequirements for ReducerContext {
    fn require_internal_access(&self) -> ServiceResult<()> {
        if !self.sender_auth().is_internal() {
            return Err(ServiceError::unauthorized(self.sender(), "Private access required"));
        }
        Ok(())
    }

    fn require_online(&self) -> ServiceResult<CharacterV1> {
        self.character_services().get_current(self.sender())
    }
}

fn validate_u64(value: u64, name: impl Into<String>, min_value: u64, max_value: u64) -> ServiceResult<()> {
    if value < min_value {
        Err(ValidationError::field_too_small(name, min_value))
    } else if value > max_value {
        Err(ValidationError::field_too_large(name, max_value))
    } else {
        Ok(())
    }
}

fn validate_str(value: &str, name: impl Into<String>, min_len: u64, max_len: u64) -> ServiceResult<()> {
    let len = value.chars().count() as u64;
    if min_len > 0 && value.is_empty() {
        Err(ValidationError::required_field(name))
    } else if len < min_len {
        Err(ValidationError::field_too_small(name, min_len))
    } else if len > max_len {
        Err(ValidationError::field_too_large(name, max_len))
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Field '{0}' is required")]
    RequiredField(String),
    #[error("Field '{0}' must be at least {1}")]
    FieldTooSmall(String, u64),
    #[error("Field '{0}' must be at most {1}")]
    FieldTooLarge(String, u64),
}

impl ValidationError {
    pub fn required_field(name: impl Into<String>) -> ServiceError {
        ValidationError::RequiredField(name.into()).map_validation_error()
    }

    pub fn field_too_small(name: impl Into<String>, min_value: u64) -> ServiceError {
        ValidationError::FieldTooSmall(name.into(), min_value).map_validation_error()
    }

    pub fn field_too_large(name: impl Into<String>, max_value: u64) -> ServiceError {
        ValidationError::FieldTooLarge(name.into(), max_value).map_validation_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_u64_accepts_values_in_range() {
        assert!(validate_u64(5, "value", 1, 10).is_ok());
    }

    #[test]
    fn validate_u64_rejects_values_below_range() {
        assert!(validate_u64(0, "value", 1, 10).is_err());
    }

    #[test]
    fn validate_str_requires_non_empty_when_min_is_positive() {
        assert!(validate_str("", "name", 1, 10).is_err());
    }
}
