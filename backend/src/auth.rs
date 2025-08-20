use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, CreateUserRequest, LoginRequest, LoginResponse, SubscriptionTier};

pub struct AuthService;

impl AuthService {
    pub async fn register(pool: &PgPool, req: CreateUserRequest) -> Result<User> {
        // Hash password
        let password_hash = Self::hash_password(&req.password)?;
        
        let user_id = Uuid::new_v4();
        
        let user = sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, subscription_tier)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, password_hash, full_name, stripe_customer_id,
                     subscription_tier, created_at, updated_at
            "#,
            user_id,
            req.email,
            password_hash,
            req.full_name,
            "free"
        )
        .fetch_one(pool)
        .await?;
        
        Ok(User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            full_name: user.full_name,
            stripe_customer_id: user.stripe_customer_id,
            subscription_tier: SubscriptionTier::Free,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
    
    pub async fn login(pool: &PgPool, req: LoginRequest) -> Result<LoginResponse> {
        let user_row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, full_name, stripe_customer_id,
                   subscription_tier, created_at, updated_at
            FROM users 
            WHERE email = $1
            "#,
            req.email
        )
        .fetch_optional(pool)
        .await?;
        
        match user_row {
            Some(user_row) => {
                if Self::verify_password(&req.password, &user_row.password_hash)? {
                    let token = Self::generate_jwt_token(&user_row.id)?;
                    let user = User {
                        id: user_row.id,
                        email: user_row.email,
                        password_hash: user_row.password_hash,
                        full_name: user_row.full_name,
                        stripe_customer_id: user_row.stripe_customer_id,
                        subscription_tier: match user_row.subscription_tier.as_str() {
                            "starter" => SubscriptionTier::Starter,
                            "professional" => SubscriptionTier::Professional,
                            "enterprise" => SubscriptionTier::Enterprise,
                            _ => SubscriptionTier::Free,
                        },
                        created_at: user_row.created_at,
                        updated_at: user_row.updated_at,
                    };
                    Ok(LoginResponse {
                        token,
                        user: user.into(),
                    })
                } else {
                    anyhow::bail!("Invalid password");
                }
            }
            None => anyhow::bail!("User not found"),
        }
    }
    
    fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();
        Ok(password_hash)
    }
    
    fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
    
    fn generate_jwt_token(_user_id: &Uuid) -> Result<String> {
        // TODO: Implement proper JWT token generation
        Ok("dummy-jwt-token".to_string())
    }
}