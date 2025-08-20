use actix_web::{web, HttpResponse, Result, Scope};

pub fn configure() -> Scope {
    web::scope("/projects")
        .route("", web::post().to(create_project))
        .route("", web::get().to(list_projects))
        .route("/{id}", web::get().to(get_project))
        .route("/{id}", web::delete().to(delete_project))
}

async fn create_project() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Create project endpoint - database not connected yet"
    })))
}

async fn list_projects() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!([])))
}

async fn get_project() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Get project endpoint - database not connected yet"
    })))
}

async fn delete_project() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete project endpoint - database not connected yet"
    })))
}