use actix_web::{web, HttpResponse, Result, Scope};
use sqlx::PgPool;
use crate::models::{CreateProjectRequest, ProjectResponse};
use crate::services::project::ProjectService;

pub fn configure() -> Scope {
    web::scope("/projects")
        .route("", web::post().to(create_project))
        .route("", web::get().to(list_projects))
        .route("/{id}", web::get().to(get_project))
        .route("/{id}", web::delete().to(delete_project))
}

async fn create_project(
    pool: web::Data<PgPool>,
    req: web::Json<CreateProjectRequest>,
) -> Result<HttpResponse> {
    // TODO: Extract user_id from JWT token
    let user_id = uuid::Uuid::new_v4(); // Placeholder
    
    match ProjectService::create_project(&pool, user_id, req.into_inner()).await {
        Ok(project) => {
            let response: ProjectResponse = project.into();
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            tracing::error!("Project creation failed: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Project creation failed",
                "message": e.to_string()
            })))
        }
    }
}

async fn list_projects(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // TODO: Extract user_id from JWT token and filter by user
    match ProjectService::list_projects(&pool, None).await {
        Ok(projects) => {
            let responses: Vec<ProjectResponse> = projects.into_iter().map(|p| p.into()).collect();
            Ok(HttpResponse::Ok().json(responses))
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve projects"
            })))
        }
    }
}

async fn get_project(
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse> {
    let project_id = path.into_inner();
    
    match ProjectService::get_project(&pool, project_id).await {
        Ok(Some(project)) => {
            let response: ProjectResponse = project.into();
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Project not found"
        }))),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve project"
            })))
        }
    }
}

async fn delete_project(
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse> {
    let project_id = path.into_inner();
    
    match ProjectService::delete_project(&pool, project_id).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Project not found"
        }))),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete project"
            })))
        }
    }
}