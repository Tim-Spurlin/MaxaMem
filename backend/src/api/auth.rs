use actix_web::{web, HttpResponse, Result, Scope};

pub fn configure() -> Scope {
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/me", web::get().to(get_current_user))
}

async fn register() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Registration endpoint - database not connected yet"
    })))
}

async fn login() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Login endpoint - database not connected yet"
    })))
}

async fn get_current_user() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Current user endpoint - authentication not implemented yet"
    })))
}