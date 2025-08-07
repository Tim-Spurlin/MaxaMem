// main.rs - Entry point for the DocGen SaaS backend
use actix_web::{web, App, HttpServer, middleware};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber;

mod api;
mod models;
mod services;
mod utils;
mod db;

use crate::services::{
    orchestrator::Orchestrator,
    github_service::GitHubService,
    openai_service::OpenAIService,
    claude_service::ClaudeService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub orchestrator: Arc<Orchestrator>,
    pub github: Arc<GitHubService>,
    pub stripe_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");
    
    // Redis connection
    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be set");
    let redis = redis::Client::open(redis_url)
        .expect("Failed to connect to Redis");
    
    // Initialize services
    let openai = Arc::new(OpenAIService::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set")
    ));
    
    let claude = Arc::new(ClaudeService::new(
        std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY must be set")
    ));
    
    let github = Arc::new(GitHubService::new(
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"),
        std::env::var("GITHUB_OWNER").expect("GITHUB_OWNER must be set"),
    ));
    
    let orchestrator = Arc::new(Orchestrator::new(
        db.clone(),
        redis.clone(),
        openai,
        claude,
        github.clone(),
    ));
    
    let app_state = AppState {
        db: db.clone(),
        redis: redis.clone(),
        orchestrator: orchestrator.clone(),
        github: github.clone(),
        stripe_secret: std::env::var("STRIPE_SECRET_KEY")
            .expect("STRIPE_SECRET_KEY must be set"),
    };
    
    // Start background job processor
    let orchestrator_clone = orchestrator.clone();
    tokio::spawn(async move {
        orchestrator_clone.start_job_processor().await;
    });
    
    info!("Starting DocGen SaaS server on port 8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .configure(api::configure)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

// api/mod.rs - API route configuration
pub mod auth;
pub mod projects;
pub mod generation;
pub mod webhooks;
pub mod subscription;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/refresh", web::post().to(auth::refresh_token))
            )
            .service(
                web::scope("/projects")
                    .route("", web::post().to(projects::create_project))
                    .route("", web::get().to(projects::list_projects))
                    .route("/{id}", web::get().to(projects::get_project))
                    .route("/{id}/status", web::get().to(projects::get_status))
                    .route("/{id}/documents", web::get().to(projects::get_documents))
            )
            .service(
                web::scope("/generation")
                    .route("/start", web::post().to(generation::start_generation))
                    .route("/{id}/retry", web::post().to(generation::retry_step))
                    .route("/{id}/cancel", web::post().to(generation::cancel))
            )
            .service(
                web::scope("/subscription")
                    .route("/plans", web::get().to(subscription::get_plans))
                    .route("/current", web::get().to(subscription::get_current))
                    .route("/upgrade", web::post().to(subscription::upgrade))
            )
    )
    .route("/webhooks/stripe", web::post().to(webhooks::stripe_webhook));
}

// services/orchestrator.rs - Main generation orchestration
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub step: GenerationStep,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStep {
    DevPlan,
    Architecture,
    Blueprint,
    Readme,
    DirectoryTree,
    CommunicationSchema,
    AgentFiles,
    GitHubScaffold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

pub struct Orchestrator {
    db: PgPool,
    redis: redis::Client,
    openai: Arc<OpenAIService>,
    claude: Arc<ClaudeService>,
    github: Arc<GitHubService>,
}

impl Orchestrator {
    pub fn new(
        db: PgPool,
        redis: redis::Client,
        openai: Arc<OpenAIService>,
        claude: Arc<ClaudeService>,
        github: Arc<GitHubService>,
    ) -> Self {
        Self { db, redis, openai, claude, github }
    }
    
    pub async fn start_generation(&self, project_id: Uuid, user_prompt: String) -> Result<(), Error> {
        // Create initial job
        let job = GenerationJob {
            id: Uuid::new_v4(),
            project_id,
            user_id: self.get_project_owner(project_id).await?,
            step: GenerationStep::DevPlan,
            status: JobStatus::Pending,
        };
        
        // Queue the job
        self.queue_job(job).await?;
        
        // Start processing
        self.process_generation(project_id, user_prompt).await?;
        
        Ok(())
    }
    
    async fn process_generation(&self, project_id: Uuid, user_prompt: String) -> Result<(), Error> {
        // Step 1: Generate Development Plan
        let dev_plan = self.generate_dev_plan(&user_prompt).await?;
        self.save_document(project_id, "dev_plan", &dev_plan).await?;
        
        // Step 2: Generate Technical Architecture
        let architecture = self.generate_architecture(&dev_plan).await?;
        self.save_document(project_id, "architecture", &architecture).await?;
        
        // Step 3: Generate Blueprint JSON
        let blueprint = self.generate_blueprint(&dev_plan, &architecture).await?;
        self.save_document(project_id, "blueprint", &blueprint).await?;
        
        // Step 4: Generate Main README
        let readme = self.generate_readme(&dev_plan, &architecture, &blueprint).await?;
        self.save_document(project_id, "readme", &readme).await?;
        
        // Step 5: Generate Directory Tree
        let tree = self.generate_tree(&blueprint).await?;
        self.save_document(project_id, "tree", &tree).await?;
        
        // Step 6: Generate Communication Schema
        let schema = self.generate_communication_schema(
            &dev_plan,
            &architecture,
            &blueprint,
            &tree
        ).await?;
        self.save_document(project_id, "schema", &schema).await?;
        
        // Step 7: Generate AGENT.md and README.md for each directory
        let agents = self.generate_agent_files(&schema).await?;
        self.save_document(project_id, "agents", &serde_json::to_string(&agents)?).await?;
        
        // Step 8: Create GitHub repository and scaffold
        let repo_name = self.get_project_name(project_id).await?;
        self.scaffold_github_repo(&repo_name, &schema, agents).await?;
        
        // Update project status
        self.update_project_status(project_id, "completed").await?;
        
        Ok(())
    }
    
    async fn generate_dev_plan(&self, prompt: &str) -> Result<String, Error> {
        let system = "You are an expert software architect. Create comprehensive development plans.";
        let user_prompt = format!(
            "Create a detailed development plan for: {}\n\
            Include: overview, tech stack, milestones, features, database schema, API endpoints, \
            security, deployment. Format as markdown.",
            prompt
        );
        
        self.openai.chat_completion(system, &user_prompt).await
    }
    
    async fn generate_architecture(&self, dev_plan: &str) -> Result<String, Error> {
        let prompt = format!(
            "Based on this development plan, create a detailed technical architecture document:\n\n{}\n\n\
            Include: system components, data flow, communication protocols, technology choices, \
            scaling considerations. Format as markdown.",
            dev_plan
        );
        
        self.openai.chat_completion(
            "You are a senior solutions architect.",
            &prompt
        ).await
    }
    
    async fn generate_blueprint(&self, dev_plan: &str, architecture: &str) -> Result<String, Error> {
        let prompt = format!(
            "Create a comprehensive blueprint.json based on:\n\
            Development Plan:\n{}\n\n\
            Architecture:\n{}\n\n\
            Generate a detailed JSON schema with all project specifications.",
            dev_plan, architecture
        );
        
        self.openai.chat_completion(
            "You are an expert at creating structured project blueprints.",
            &prompt
        ).await
    }
    
    async fn generate_readme(&self, dev_plan: &str, arch: &str, blueprint: &str) -> Result<String, Error> {
        let prompt = format!(
            "Create a comprehensive README.md with visual diagrams (mermaid) based on:\n\
            Dev Plan:\n{}\n\nArchitecture:\n{}\n\nBlueprint:\n{}\n\n\
            Include: executive summary, features, architecture diagrams, setup instructions, \
            API documentation, deployment guide.",
            dev_plan, arch, blueprint
        );
        
        self.claude.generate(&prompt).await
    }
    
    async fn generate_communication_schema(
        &self,
        dev_plan: &str,
        arch: &str,
        blueprint: &str,
        tree: &str
    ) -> Result<String, Error> {
        let prompt = format!(
            "Generate a comprehensive communication schema JSON that maps all component interactions.\n\n\
            Development Plan:\n{}\n\n\
            Architecture:\n{}\n\n\
            Blueprint:\n{}\n\n\
            Directory Tree:\n{}\n\n\
            Create a schema with:\n\
            1. Global communication protocols\n\
            2. Complete directory structure with criticality scores (1-10)\n\
            3. Event flows\n\
            4. Communication matrix\n\
            5. Platform-specific details\n\
            6. Error handling patterns\n\n\
            Each directory and file should have:\n\
            - Criticality score\n\
            - Communication patterns\n\
            - Dependencies\n\
            - Triggers\n\
            - Protocol details",
            dev_plan, arch, blueprint, tree
        );
        
        self.claude.generate(&prompt).await
    }
    
    async fn generate_agent_files(&self, schema: &str) -> Result<Vec<AgentFile>, Error> {
        let schema: CommunicationSchema = serde_json::from_str(schema)?;
        let mut agent_files = Vec::new();
        
        for (dir_path, dir_config) in schema.directory_structure.iter() {
            let content = self.generate_directory_docs(dir_path, dir_config, &schema)?;
            
            agent_files.push(AgentFile {
                path: format!("{}/README.md", dir_path.trim_end_matches('/')),
                content: content.clone(),
            });
            
            agent_files.push(AgentFile {
                path: format!("{}/AGENT.md", dir_path.trim_end_matches('/')),
                content,
            });
        }
        
        Ok(agent_files)
    }
    
    fn generate_directory_docs(
        &self,
        dir_path: &str,
        dir_config: &DirectoryConfig,
        schema: &CommunicationSchema
    ) -> Result<String, Error> {
        let mut content = format!(
            "# {} - {}\nCriticality: {}/10\n\n",
            dir_path,
            dir_config.description,
            dir_config.criticality
        );
        
        // Sort files by criticality
        let mut files: Vec<_> = dir_config.files.iter().collect();
        files.sort_by(|a, b| b.1.criticality.cmp(&a.1.criticality));
        
        // Critical files section
        content.push_str("## Critical Files (Must maintain for system stability)\n");
        for (name, file) in files.iter().filter(|(_, f)| f.criticality >= 9) {
            content.push_str(&format!(
                "### {}\n- **Criticality:** {}/10\n- **Type:** {}\n- **Purpose:** {}\n\
                - **Communicates with:**\n",
                name, file.criticality, file.file_type, file.purpose
            ));
            
            if let Some(comms) = &file.communicates {
                for (target, details) in comms {
                    content.push_str(&format!("  - {}: {}\n", target, details));
                }
            }
            
            if !file.dependencies.is_empty() {
                content.push_str(&format!("- **Dependencies:** {:?}\n", file.dependencies));
            }
            
            content.push_str("\n");
        }
        
        // Important files section
        content.push_str("\n## Important Files (Breaking these affects functionality)\n");
        for (name, file) in files.iter().filter(|(_, f)| f.criticality >= 7 && f.criticality < 9) {
            content.push_str(&format!(
                "- **{}** (Criticality: {}/10): {}\n",
                name, file.criticality, file.purpose
            ));
        }
        
        // Supporting files section
        if files.iter().any(|(_, f)| f.criticality < 7) {
            content.push_str("\n## Supporting Files (Can be modified with care)\n");
            for (name, file) in files.iter().filter(|(_, f)| f.criticality < 7) {
                content.push_str(&format!(
                    "- **{}** (Criticality: {}/10): {}\n",
                    name, file.criticality, file.purpose
                ));
            }
        }
        
        // Communication patterns
        content.push_str("\n## Communication Patterns\n");
        if let Some(receives) = &dir_config.receives_from {
            content.push_str(&format!("- **Receives from:** {:?}\n", receives));
        }
        if let Some(sends) = &dir_config.sends_to {
            content.push_str(&format!("- **Sends to:** {:?}\n", sends));
        }
        if let Some(protocols) = &dir_config.protocols {
            content.push_str(&format!("- **Protocols:** {:?}\n", protocols));
        }
        
        // File relationships matrix
        content.push_str("\n## File Relationships\n```json\n");
        let relationships = self.build_relationships(&dir_config.files);
        content.push_str(&serde_json::to_string_pretty(&relationships)?);
        content.push_str("\n```\n");
        
        // Event flows if applicable
        if let Some(flows) = schema.event_flows.get(dir_path) {
            content.push_str("\n## Event Flows\n");
            for flow in flows {
                content.push_str(&format!("- {}: {}\n", flow.name, flow.description));
            }
        }
        
        Ok(content)
    }
}

// models/schema.rs - Data structures for communication schema
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationSchema {
    pub version: String,
    pub project_name: String,
    pub description: String,
    pub global_communication_protocols: GlobalProtocols,
    pub directory_structure: HashMap<String, DirectoryConfig>,
    pub event_flows: HashMap<String, Vec<EventFlow>>,
    pub communication_matrix: CommunicationMatrix,
    pub platform_specific: PlatformSpecific,
    pub error_propagation: ErrorPropagation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub criticality: u8,
    pub description: String,
    pub files: HashMap<String, FileConfig>,
    pub directories: Option<HashMap<String, DirectoryConfig>>,
    pub receives_from: Option<Vec<String>>,
    pub sends_to: Option<Vec<String>>,
    pub protocols: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub criticality: u8,
    #[serde(rename = "type")]
    pub file_type: String,
    pub purpose: String,
    pub dependencies: Vec<String>,
    pub communicates: Option<HashMap<String, String>>,
    pub triggers: Option<Vec<String>>,
    pub modifies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFile {
    pub path: String,
    pub content: String,
}

// services/github_service.rs - GitHub integration
use octocrab::{Octocrab, models::repos::Repository};
use base64::{Engine as _, engine::general_purpose};

pub struct GitHubService {
    client: Octocrab,
    owner: String,
}

impl GitHubService {
    pub fn new(token: String, owner: String) -> Self {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .expect("Failed to build GitHub client");
        
        Self { client, owner }
    }
    
    pub async fn create_repository(
        &self,
        name: &str,
        description: &str,
        private: bool
    ) -> Result<Repository, Error> {
        let repo = self.client
            .repos(&self.owner, name)
            .create()
            .description(description)
            .private(private)
            .auto_init(true)
            .send()
            .await?;
        
        Ok(repo)
    }
    
    pub async fn create_file(
        &self,
        repo: &str,
        path: &str,
        content: &str,
        message: &str
    ) -> Result<(), Error> {
        let encoded = general_purpose::STANDARD.encode(content);
        
        self.client
            .repos(&self.owner, repo)
            .create_file(path, message, encoded)
            .branch("main")
            .send()
            .await?;
        
        Ok(())
    }
    
    pub async fn create_directory_structure(
        &self,
        repo: &str,
        files: Vec<AgentFile>
    ) -> Result<(), Error> {
        // Create files in batches to avoid rate limits
        for chunk in files.chunks(10) {
            for file in chunk {
                self.create_file(
                    repo,
                    &file.path,
                    &file.content,
                    &format!("Add {}", file.path)
                ).await?;
                
                // Small delay to respect rate limits
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        Ok(())
    }
}
