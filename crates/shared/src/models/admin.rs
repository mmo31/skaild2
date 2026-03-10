use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Admin user account for control-plane access
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Admin {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new admin
#[derive(Debug, Clone)]
pub struct CreateAdminInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Validation errors for admin operations
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("Email is required and must be valid")]
    InvalidEmail,
    
    #[error("Password must be at least 12 characters")]
    PasswordTooShort,
    
    #[error("Password must contain at least one uppercase letter")]
    PasswordNeedsUppercase,
    
    #[error("Password must contain at least one lowercase letter")]
    PasswordNeedsLowercase,
    
    #[error("Password must contain at least one number")]
    PasswordNeedsNumber,
    
    #[error("Password must contain at least one special character")]
    PasswordNeedsSpecial,
    
    #[error("Password hashing failed")]
    HashingFailed,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Hash a password using argon2
pub fn hash_password(password: &str) -> Result<String, AdminError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AdminError::HashingFailed)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AdminError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| AdminError::InvalidCredentials)?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), AdminError> {
    if password.len() < 12 {
        return Err(AdminError::PasswordTooShort);
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AdminError::PasswordNeedsUppercase);
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(AdminError::PasswordNeedsLowercase);
    }
    
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(AdminError::PasswordNeedsNumber);
    }
    
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AdminError::PasswordNeedsSpecial);
    }
    
    Ok(())
}

/// Validate email format (basic check)
pub fn validate_email(email: &str) -> Result<(), AdminError> {
    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(AdminError::InvalidEmail);
    }
    Ok(())
}

/// Create a new admin in the database
pub async fn create_admin(
    pool: &crate::db::DbPool, 
    input: CreateAdminInput
) -> Result<Admin, AdminError> {
    // Validate input
    validate_email(&input.email)?;
    validate_password(&input.password)?;
    
    // Hash password
    let password_hash = hash_password(&input.password)?;
    
    // Insert into database
    let admin = sqlx::query_as::<_, Admin>(
        r#"
        INSERT INTO admins (name, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, name, email, password_hash, created_at, updated_at
        "#
    )
    .bind(&input.name)
    .bind(&input.email)
    .bind(&password_hash)
    .fetch_one(pool)
    .await
    .map_err(|e| AdminError::DatabaseError(e.to_string()))?;
    
    Ok(admin)
}

/// Authenticate an admin by email and password
pub async fn authenticate_admin(
    pool: &crate::db::DbPool,
    email: &str,
    password: &str,
) -> Result<Admin, AdminError> {
    // Fetch admin by email
    let admin = sqlx::query_as::<_, Admin>(
        r#"
        SELECT id, name, email, password_hash, created_at, updated_at
        FROM admins
        WHERE email = $1
        "#
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|e| AdminError::DatabaseError(e.to_string()))?
    .ok_or(AdminError::InvalidCredentials)?;
    
    // Verify password
    if verify_password(password, &admin.password_hash)? {
        Ok(admin)
    } else {
        Err(AdminError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation_too_short() {
        assert!(matches!(
            validate_password("Short1!"),
            Err(AdminError::PasswordTooShort)
        ));
    }

    #[test]
    fn test_password_validation_no_uppercase() {
        assert!(matches!(
            validate_password("nouppercase123!"),
            Err(AdminError::PasswordNeedsUppercase)
        ));
    }

    #[test]
    fn test_password_validation_no_lowercase() {
        assert!(matches!(
            validate_password("NOLOWERCASE123!"),
            Err(AdminError::PasswordNeedsLowercase)
        ));
    }

    #[test]
    fn test_password_validation_no_number() {
        assert!(matches!(
            validate_password("NoNumberHere!"),
            Err(AdminError::PasswordNeedsNumber)
        ));
    }

    #[test]
    fn test_password_validation_no_special() {
        assert!(matches!(
            validate_password("NoSpecialChar1"),
            Err(AdminError::PasswordNeedsSpecial)
        ));
    }

    #[test]
    fn test_password_validation_valid() {
        assert!(validate_password("SecurePassword123!").is_ok());
    }

    #[test]
    fn test_email_validation_invalid() {
        assert!(matches!(validate_email(""), Err(AdminError::InvalidEmail)));
        assert!(matches!(validate_email("notanemail"), Err(AdminError::InvalidEmail)));
        assert!(matches!(validate_email("no@domain"), Err(AdminError::InvalidEmail)));
    }

    #[test]
    fn test_email_validation_valid() {
        assert!(validate_email("user@example.com").is_ok());
    }

    #[test]
    fn test_password_hashing() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();
        
        // Hash should not be empty
        assert!(!hash.is_empty());
    }
        
    #[test]
    fn test_password_verification() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();
        
        // Correct password should verify
        assert!(verify_password(password, &hash).unwrap());
        
        // Wrong password should not verify
        assert!(!verify_password("WrongPassword123!", &hash).unwrap());
    }

    #[test]
    fn test_password_hashing_produces_different_hashes() {
        let password = "SecurePassword123!";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        // Same password should produce different hashes (due to salt)
        assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
