//! Validation utilities and custom validators.

use crate::{NexoraError, NexoraResult};
use validator::{Validate, ValidationError, ValidationErrors};

/// Validate email format.
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains('@') || email.len() > 254 {
        return Err(ValidationError::new("invalid_email"));
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::new("invalid_email"));
    }
    Ok(())
}

/// Validate phone number (E.164 format).
pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    if !phone.starts_with('+') || phone.len() < 8 || phone.len() > 15 {
        return Err(ValidationError::new("invalid_phone"));
    }
    if !phone.chars().skip(1).all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("invalid_phone"));
    }
    Ok(())
}

/// Validate slug format (lowercase, alphanumeric, hyphens).
pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if slug.is_empty() || slug.len() > 63 {
        return Err(ValidationError::new("invalid_slug"));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ValidationError::new("invalid_slug"));
    }
    if slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err(ValidationError::new("invalid_slug"));
    }
    Ok(())
}

/// Validate currency code (ISO 4217).
pub fn validate_currency(currency: &str) -> Result<(), ValidationError> {
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new("invalid_currency"));
    }
    Ok(())
}

/// Validate country code (ISO 3166-1 alpha-2).
pub fn validate_country_code(code: &str) -> Result<(), ValidationError> {
    if code.len() != 2 || !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new("invalid_country_code"));
    }
    Ok(())
}

/// Validate ULID format.
pub fn validate_ulid(ulid: &str) -> Result<(), ValidationError> {
    ulid.parse::<crate::ulid::Ulid>()
        .map(|_| ())
        .map_err(|_| ValidationError::new("invalid_ulid"))
}

/// Validate password strength.
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 12 {
        return Err(ValidationError::new("password_too_short"));
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::new("password_needs_lowercase"));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new("password_needs_uppercase"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_needs_digit"));
    }
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        return Err(ValidationError::new("password_needs_special"));
    }
    Ok(())
}

/// Trait for domain-specific validation.
pub trait DomainValidate {
    fn domain_validate(&self) -> NexoraResult<()>;
}

/// Combine validator errors into NexoraError.
pub fn combine_errors(errors: ValidationErrors) -> NexoraError {
    let messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                let msg = e.message.as_deref().unwrap_or("invalid value");
                format!("{field}: {msg}")
            })
        })
        .collect();
    NexoraError::Validation(messages.join("; "))
}

/// Validated wrapper for types that implement Validate.
pub struct Validated<T>(pub T);

impl<T: Validate> Validated<T> {
    pub fn new(value: T) -> NexoraResult<Self> {
        value.validate().map_err(combine_errors)?;
        Ok(Self(value))
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
