use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Project, CreateProjectRequest, ProjectStatus};

pub struct ProjectService;

impl ProjectService {
    pub async fn create_project(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateProjectRequest,
    ) -> Result<Project> {
        let project_id = Uuid::new_v4();
        
        let project_row = sqlx::query!(
            r#"
            INSERT INTO projects (id, user_id, name, description, status, progress, technologies)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, name, description, status, progress, repository_url, technologies, created_at, updated_at
            "#,
            project_id,
            user_id,
            req.name,
            req.description,
            "pending",
            0i32,
            &req.technologies
        )
        .fetch_one(pool)
        .await?;
        
        Ok(Project {
            id: project_row.id,
            user_id: project_row.user_id,
            name: project_row.name,
            description: project_row.description,
            status: ProjectStatus::Pending,
            progress: project_row.progress,
            repository_url: project_row.repository_url,
            technologies: project_row.technologies,
            created_at: project_row.created_at,
            updated_at: project_row.updated_at,
        })
    }
    
    pub async fn list_projects(pool: &PgPool, user_id: Option<Uuid>) -> Result<Vec<Project>> {
        let project_rows = if let Some(user_id) = user_id {
            sqlx::query!(
                r#"
                SELECT id, user_id, name, description, status, progress, repository_url, technologies, created_at, updated_at
                FROM projects 
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT id, user_id, name, description, status, progress, repository_url, technologies, created_at, updated_at
                FROM projects 
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(pool)
            .await?
        };
        
        let projects = project_rows
            .into_iter()
            .map(|row| Project {
                id: row.id,
                user_id: row.user_id,
                name: row.name,
                description: row.description,
                status: match row.status.as_str() {
                    "generating" => ProjectStatus::Generating,
                    "complete" => ProjectStatus::Complete,
                    "failed" => ProjectStatus::Failed,
                    _ => ProjectStatus::Pending,
                },
                progress: row.progress,
                repository_url: row.repository_url,
                technologies: row.technologies,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();
        
        Ok(projects)
    }
    
    pub async fn get_project(pool: &PgPool, project_id: Uuid) -> Result<Option<Project>> {
        let project_row = sqlx::query!(
            r#"
            SELECT id, user_id, name, description, status, progress, repository_url, technologies, created_at, updated_at
            FROM projects 
            WHERE id = $1
            "#,
            project_id
        )
        .fetch_optional(pool)
        .await?;
        
        Ok(project_row.map(|row| Project {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            description: row.description,
            status: match row.status.as_str() {
                "generating" => ProjectStatus::Generating,
                "complete" => ProjectStatus::Complete,
                "failed" => ProjectStatus::Failed,
                _ => ProjectStatus::Pending,
            },
            progress: row.progress,
            repository_url: row.repository_url,
            technologies: row.technologies,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }
    
    pub async fn delete_project(pool: &PgPool, project_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM projects WHERE id = $1",
            project_id
        )
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn update_project_status(
        pool: &PgPool,
        project_id: Uuid,
        status: ProjectStatus,
        progress: Option<i32>,
    ) -> Result<()> {
        let status_str = match status {
            ProjectStatus::Pending => "pending",
            ProjectStatus::Generating => "generating",
            ProjectStatus::Complete => "complete",
            ProjectStatus::Failed => "failed",
        };
        
        if let Some(progress) = progress {
            sqlx::query!(
                r#"
                UPDATE projects 
                SET status = $1, progress = $2, updated_at = NOW()
                WHERE id = $3
                "#,
                status_str,
                progress,
                project_id
            )
            .execute(pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                UPDATE projects 
                SET status = $1, updated_at = NOW()
                WHERE id = $2
                "#,
                status_str,
                project_id
            )
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }
}