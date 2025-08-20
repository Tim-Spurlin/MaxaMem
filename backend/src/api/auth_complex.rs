use actix_web::{web, HttpResponse, Result, Scope};
use sqlx::PgPool;
use crate::models::{CreateUserRequest, LoginRequest, LoginResponse, UserResponse};
use crate::auth::AuthService;

pub fn configure() -> Scope {
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/me", web::get().to(get_current_user))
}

async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    match AuthService::register(&pool, req.into_inner()).await {
        Ok(user) => {
            let response: UserResponse = user.into();
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            tracing::error!("Registration failed: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Registration failed",
                "message": e.to_string()
            })))
        }
    }
}

async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    match AuthService::login(&pool, req.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            tracing::error!("Login failed: {}", e);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Login failed",
                "message": "Invalid credentials"
            })))
        }
    }
}

async fn get_current_user() -> Result<HttpResponse> {
    // TODO: Implement JWT token validation and user retrieval
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "JWT authentication not yet implemented"
    })))
}