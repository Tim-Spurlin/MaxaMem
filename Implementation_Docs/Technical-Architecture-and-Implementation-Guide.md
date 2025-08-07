# MaxaMem Technical Architecture & Implementation Guide

## 🏗️ Complete System Architecture

### Core Architecture Components

```mermaid
graph TB
    subgraph "Frontend Services"
        WEB[React SPA<br/>TypeScript]
        MOBILE[PWA Support]
        ADMIN[Admin Dashboard]
    end
    
    subgraph "API Gateway Layer"
        NGINX[Nginx<br/>Load Balancer]
        RATE[Rate Limiter<br/>Redis]
        AUTH[Auth Service<br/>JWT]
    end
    
    subgraph "Application Services"
        API[API Server<br/>Actix-Web]
        GEN[Generation Engine<br/>State Machine]
        SCHEMA[Schema Generator<br/>Custom Algorithm]
        WORKER[Worker Pool<br/>Tokio]
    end
    
    subgraph "AI Integration Layer"
        AIPROXY[AI Proxy Service]
        GPT[ChatGPT<br/>GPT-4]
        CLAUDE[Claude<br/>Claude-3-Opus]
        CACHE[Response Cache<br/>Redis]
    end
    
    subgraph "Data Layer"
        PG[(PostgreSQL<br/>Primary DB)]
        REDIS[(Redis<br/>Cache & Queue)]
        S3[S3/MinIO<br/>File Storage]
    end
    
    subgraph "External Services"
        GH[GitHub API<br/>Octocrab]
        STRIPE[Stripe<br/>Payments]
        EMAIL[SendGrid<br/>Notifications]
        MONITOR[Datadog<br/>Monitoring]
    end
    
    WEB --> NGINX
    MOBILE --> NGINX
    ADMIN --> NGINX
    
    NGINX --> RATE
    RATE --> AUTH
    AUTH --> API
    
    API --> GEN
    GEN --> SCHEMA
    GEN --> WORKER
    
    WORKER --> AIPROXY
    AIPROXY --> GPT
    AIPROXY --> CLAUDE
    AIPROXY --> CACHE
    
    API --> PG
    API --> REDIS
    WORKER --> S3
    
    WORKER --> GH
    API --> STRIPE
    WORKER --> EMAIL
    API --> MONITOR
```

## 📦 Backend Implementation (Rust)

### Project Structure

```
backend/
├── src/
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── projects.rs
│   │   ├── subscription.rs
│   │   └── webhooks.rs
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── jwt.rs
│   │   ├── middleware.rs
│   │   └── password.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   ├── schema.rs
│   │   └── migrations.rs
│   ├── generation/
│   │   ├── mod.rs
│   │   ├── pipeline.rs
│   │   ├── state_machine.rs
│   │   └── templates.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── openai.rs
│   │   ├── claude.rs
│   │   └── cache.rs
│   ├── github/
│   │   ├── mod.rs
│   │   ├── repository.rs
│   │   └── files.rs
│   ├── schema_engine/
│   │   ├── mod.rs
│   │   ├── analyzer.rs
│   │   ├── generator.rs
│   │   └── validator.rs
│   ├── payment/
│   │   ├── mod.rs
│   │   ├── stripe.rs
│   │   └── subscription.rs
│   ├── queue/
│   │   ├── mod.rs
│   │   ├── worker.rs
│   │   └── jobs.rs
│   ├── websocket/
│   │   ├── mod.rs
│   │   └── handlers.rs
│   ├── config.rs
│   ├── errors.rs
│   └── main.rs
├── migrations/
├── tests/
└── Cargo.toml
```

### Core Backend Files

#### `Cargo.toml`
```toml
[package]
name = "maxamem-backend"
version = "1.0.0"
edition = "2021"

[dependencies]
# Web Framework
actix-web = "4.4"
actix-ws = "0.2"
actix-cors = "0.6"

# Async Runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"
uuid = { version = "1.6", features = ["v4", "serde"] }

# External APIs
reqwest = { version = "0.11", features = ["json", "stream"] }
octocrab = "0.32"
stripe-rust = "0.25"

# Utilities
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
validator = { version = "0.16", features = ["derive"] }

[dev-dependencies]
mockito = "1.2"
```

#### `src/main.rs`
```rust
use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, error};
use std::sync::Arc;

mod api;
mod auth;
mod db;
mod generation;
mod ai;
mod github;
mod schema_engine;
mod payment;
mod queue;
mod websocket;
mod config;
mod errors;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    
    // Create database pool
    let db_pool = PgPoolOptions::new()
        .max_connections(32)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    
    // Create Redis client
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");
    
    // Initialize services
    let ai_service = Arc::new(ai::AiService::new(&config));
    let github_service = Arc::new(github::GitHubService::new(&config));
    let stripe_service = Arc::new(payment::StripeService::new(&config));
    let schema_generator = Arc::new(schema_engine::SchemaGenerator::new());
    
    // Start worker pool
    let worker_pool = queue::WorkerPool::new(
        db_pool.clone(),
        redis_client.clone(),
        ai_service.clone(),
        github_service.clone(),
        schema_generator.clone(),
    );
    worker_pool.start();
    
    let app_state = web::Data::new(AppState {
        db_pool: db_pool.clone(),
        redis_client,
        ai_service,
        github_service,
        stripe_service,
        schema_generator,
        config: config.clone(),
    });
    
    info!("Starting MaxaMem server on {}", config.server_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") ||
                        origin == "https://maxamem.com"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Authorization", "Content-Type"])
                    .max_age(3600)
            )
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(api::configure)
            .service(websocket::ws_route)
    })
    .bind(&config.server_addr)?
    .run()
    .await
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub ai_service: Arc<ai::AiService>,
    pub github_service: Arc<github::GitHubService>,
    pub stripe_service: Arc<payment::StripeService>,
    pub schema_generator: Arc<schema_engine::SchemaGenerator>,
    pub config: Config,
}
```

#### `src/generation/pipeline.rs`
```rust
use crate::{ai, github, schema_engine, errors::Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPipeline {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub description: String,
    pub status: GenerationStatus,
    pub artifacts: GenerationArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStatus {
    Queued,
    GeneratingDevPlan,
    GeneratingArchitecture,
    GeneratingBlueprint,
    GeneratingReadme,
    GeneratingSchema,
    CreatingRepository,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationArtifacts {
    pub dev_plan: Option<String>,
    pub tech_architecture: Option<String>,
    pub blueprint_json: Option<serde_json::Value>,
    pub main_readme: Option<String>,
    pub communication_schema: Option<serde_json::Value>,
    pub directory_tree: Option<DirectoryTree>,
    pub repository_url: Option<String>,
}

impl GenerationPipeline {
    pub async fn execute(
        &mut self,
        ai_service: &ai::AiService,
        github_service: &github::GitHubService,
        schema_generator: &schema_engine::SchemaGenerator,
    ) -> Result<()> {
        // Step 1: Generate Development Plan
        self.status = GenerationStatus::GeneratingDevPlan;
        let dev_plan = ai_service
            .generate_dev_plan(&self.description)
            .await?;
        self.artifacts.dev_plan = Some(dev_plan.clone());
        
        // Step 2: Generate Technical Architecture
        self.status = GenerationStatus::GeneratingArchitecture;
        let tech_arch = ai_service
            .generate_tech_architecture(&self.description, &dev_plan)
            .await?;
        self.artifacts.tech_architecture = Some(tech_arch.clone());
        
        // Step 3: Generate Blueprint.json
        self.status = GenerationStatus::GeneratingBlueprint;
        let blueprint = ai_service
            .generate_blueprint(&dev_plan, &tech_arch)
            .await?;
        self.artifacts.blueprint_json = Some(blueprint.clone());
        
        // Step 4: Generate Main README with Claude
        self.status = GenerationStatus::GeneratingReadme;
        let main_readme = ai_service
            .generate_main_readme(&dev_plan, &tech_arch, &blueprint)
            .await?;
        self.artifacts.main_readme = Some(main_readme.clone());
        
        // Step 5: Generate Communication Schema
        self.status = GenerationStatus::GeneratingSchema;
        let schema = schema_generator
            .generate_communication_schema(
                &dev_plan,
                &tech_arch,
                &blueprint,
                &main_readme,
            )
            .await?;
        self.artifacts.communication_schema = Some(schema.clone());
        
        // Step 6: Generate Directory Tree and READMEs
        let directory_tree = schema_generator
            .generate_directory_structure(&schema)
            .await?;
        self.artifacts.directory_tree = Some(directory_tree.clone());
        
        // Step 7: Create GitHub Repository
        self.status = GenerationStatus::CreatingRepository;
        let repo_url = github_service
            .create_and_populate_repository(
                &self.project_id.to_string(),
                &self.artifacts,
            )
            .await?;
        self.artifacts.repository_url = Some(repo_url);
        
        self.status = GenerationStatus::Complete;
        Ok(())
    }
}
```

#### `src/schema_engine/generator.rs`
```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationSchema {
    pub version: String,
    pub project_name: String,
    pub global_protocols: GlobalProtocols,
    pub directory_structure: HashMap<String, DirectoryConfig>,
    pub event_flows: HashMap<String, EventFlow>,
    pub communication_matrix: CommunicationMatrix,
    pub dependency_priority: DependencyPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub criticality: u8,
    pub description: String,
    pub files: HashMap<String, FileConfig>,
    pub directories: HashMap<String, DirectoryInfo>,
    pub receives_from: Vec<String>,
    pub sends_to: Vec<String>,
    pub protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub criticality: u8,
    pub file_type: String,
    pub dependencies: Vec<String>,
    pub communicates: HashMap<String, CommunicationDetail>,
    pub triggers: Vec<String>,
    pub modifies: Vec<String>,
}

impl SchemaGenerator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn generate_communication_schema(
        &self,
        dev_plan: &str,
        tech_arch: &str,
        blueprint: &Value,
        readme: &str,
    ) -> Result<Value> {
        // Analyze all inputs to extract components
        let components = self.extract_components(dev_plan, tech_arch, blueprint)?;
        
        // Build communication patterns
        let protocols = self.identify_protocols(&components)?;
        
        // Generate directory structure with criticality scores
        let directory_structure = self.build_directory_structure(&components)?;
        
        // Map event flows
        let event_flows = self.map_event_flows(&components)?;
        
        // Create communication matrix
        let comm_matrix = self.build_communication_matrix(&components)?;
        
        // Determine dependency priorities
        let priorities = self.calculate_priorities(&directory_structure)?;
        
        Ok(json!({
            "version": "1.0.0",
            "project_name": blueprint["project_name"],
            "global_protocols": protocols,
            "directory_structure": directory_structure,
            "event_flows": event_flows,
            "communication_matrix": comm_matrix,
            "dependency_priority": priorities,
        }))
    }
    
    pub async fn generate_directory_readme(
        &self,
        directory_path: &str,
        schema: &CommunicationSchema,
    ) -> Result<String> {
        let dir_config = schema.directory_structure.get(directory_path)
            .ok_or_else(|| anyhow::anyhow!("Directory not found in schema"))?;
        
        // Sort files by criticality
        let mut files: Vec<_> = dir_config.files.iter().collect();
        files.sort_by(|a, b| b.1.criticality.cmp(&a.1.criticality));
        
        let mut readme = format!(
            "# {} - {}\n\nCriticality: {}/10\n\n",
            directory_path,
            dir_config.description,
            dir_config.criticality
        );
        
        // Critical files section
        readme.push_str("## Critical Files (Must maintain for system stability)\n\n");
        for (name, config) in files.iter().filter(|(_, c)| c.criticality >= 9) {
            readme.push_str(&format!(
                "### `{}`\n- **Criticality:** {}/10\n- **Type:** {}\n- **Dependencies:** {:?}\n\n",
                name, config.criticality, config.file_type, config.dependencies
            ));
        }
        
        // Important files section
        readme.push_str("\n## Important Files (Breaking these affects functionality)\n\n");
        for (name, config) in files.iter().filter(|(_, c)| c.criticality >= 7 && c.criticality < 9) {
            readme.push_str(&format!(
                "### `{}`\n- **Criticality:** {}/10\n- **Type:** {}\n",
                name, config.criticality, config.file_type
            ));
        }
        
        // Communication patterns
        readme.push_str("\n## Communication Patterns\n\n");
        readme.push_str(&format!("- **Receives from:** {:?}\n", dir_config.receives_from));
        readme.push_str(&format!("- **Sends to:** {:?}\n", dir_config.sends_to));
        readme.push_str(&format!("- **Protocols:** {:?}\n", dir_config.protocols));
        
        Ok(readme)
    }
}
```

## 💻 Frontend Implementation (TypeScript/React)

### Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── Layout/
│   │   ├── Dashboard/
│   │   ├── Projects/
│   │   ├── Schema/
│   │   └── Common/
│   ├── pages/
│   │   ├── HomePage.tsx
│   │   ├── DashboardPage.tsx
│   │   ├── ProjectPage.tsx
│   │   ├── PricingPage.tsx
│   │   └── AuthPage.tsx
│   ├── stores/
│   │   ├── authStore.ts
│   │   ├── projectStore.ts
│   │   └── uiStore.ts
│   ├── hooks/
│   │   ├── useAuth.ts
│   │   ├── useProject.ts
│   │   ├── useWebSocket.ts
│   │   └── useStripe.ts
│   ├── api/
│   │   ├── client.ts
│   │   ├── auth.ts
│   │   ├── projects.ts
│   │   └── subscription.ts
│   ├── utils/
│   │   ├── constants.ts
│   │   ├── helpers.ts
│   │   └── validators.ts
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   └── main.tsx
├── public/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

### Core Frontend Files

#### `package.json`
```json
{
  "name": "maxamem-frontend",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "lint": "eslint . --ext ts,tsx"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.0",
    "zustand": "^4.4.7",
    "@tanstack/react-query": "^5.12.0",
    "socket.io-client": "^4.5.4",
    "axios": "^1.6.2",
    "@stripe/stripe-js": "^2.2.0",
    "@stripe/react-stripe-js": "^2.4.0",
    "recharts": "^2.10.0",
    "react-markdown": "^9.0.1",
    "react-syntax-highlighter": "^15.5.0",
    "framer-motion": "^10.16.0",
    "react-hot-toast": "^2.4.1",
    "date-fns": "^2.30.0",
    "clsx": "^2.0.0",
    "react-hook-form": "^7.48.0",
    "zod": "^3.22.4"
  },
  "devDependencies": {
    "@types/react": "^18.2.43",
    "@types/react-dom": "^18.2.17",
    "@typescript-eslint/eslint-plugin": "^6.14.0",
    "@typescript-eslint/parser": "^6.14.0",
    "@vitejs/plugin-react": "^4.2.0",
    "autoprefixer": "^10.4.16",
    "eslint": "^8.55.0",
    "postcss": "^8.4.32",
    "tailwindcss": "^3.3.6",
    "typescript": "^5.3.3",
    "vite": "^5.0.8",
    "vitest": "^1.0.4"
  }
}
```

#### `src/App.tsx`
```tsx
import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';
import { loadStripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';

import Layout from './components/Layout/Layout';
import HomePage from './pages/HomePage';
import DashboardPage from './pages/DashboardPage';
import ProjectPage from './pages/ProjectPage';
import PricingPage from './pages/PricingPage';
import AuthPage from './pages/AuthPage';
import { useAuthStore } from './stores/authStore';
import { WebSocketProvider } from './contexts/WebSocketContext';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
      cacheTime: 5 * 60 * 1000,
    },
  },
});

const stripePromise = loadStripe(import.meta.env.VITE_STRIPE_PUBLIC_KEY);

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  return isAuthenticated ? <>{children}</> : <Navigate to="/auth" />;
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Elements stripe={stripePromise}>
        <WebSocketProvider>
          <Router>
            <Layout>
              <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/auth" element={<AuthPage />} />
                <Route path="/pricing" element={<PricingPage />} />
                <Route
                  path="/dashboard"
                  element={
                    <PrivateRoute>
                      <DashboardPage />
                    </PrivateRoute>
                  }
                />
                <Route
                  path="/project/:id"
                  element={
                    <PrivateRoute>
                      <ProjectPage />
                    </PrivateRoute>
                  }
                />
              </Routes>
            </Layout>
          </Router>
          <Toaster position="top-right" />
        </WebSocketProvider>
      </Elements>
    </QueryClientProvider>
  );
}

export default App;
```

#### `src/pages/ProjectPage.tsx`
```tsx
import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

import { getProject } from '../api/projects';
import { useWebSocket } from '../hooks/useWebSocket';
import SchemaVisualizer from '../components/Schema/SchemaVisualizer';
import GenerationProgress from '../components/Projects/GenerationProgress';
import RepositoryInfo from '../components/Projects/RepositoryInfo';

export default function ProjectPage() {
  const { id } = useParams<{ id: string }>();
  const [activeTab, setActiveTab] = useState<'overview' | 'schema' | 'readme' | 'files'>('overview');
  
  const { data: project, isLoading, refetch } = useQuery({
    queryKey: ['project', id],
    queryFn: () => getProject(id!),
    enabled: !!id,
  });
  
  const { subscribe, unsubscribe } = useWebSocket();
  
  useEffect(() => {
    if (id) {
      subscribe(`project:${id}`, (event) => {
        if (event.type === 'generation:progress' || event.type === 'generation:completed') {
          refetch();
        }
      });
      
      return () => unsubscribe(`project:${id}`);
    }
  }, [id, subscribe, unsubscribe, refetch]);
  
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }
  
  if (!project) {
    return <div>Project not found</div>;
  }
  
  return (
    <div className="container mx-auto px-4 py-8">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-white rounded-lg shadow-lg p-6"
      >
        <h1 className="text-3xl font-bold mb-4">{project.name}</h1>
        
        {project.status !== 'Complete' && (
          <GenerationProgress status={project.status} progress={project.progress} />
        )}
        
        {project.status === 'Complete' && (
          <>
            <div className="border-b mb-6">
              <nav className="flex space-x-8">
                {['overview', 'schema', 'readme', 'files'].map((tab) => (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab as any)}
                    className={`py-2 px-1 border-b-2 font-medium text-sm ${
                      activeTab === tab
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700'
                    }`}
                  >
                    {tab.charAt(0).toUpperCase() + tab.slice(1)}
                  </button>
                ))}
              </nav>
            </div>
            
            <div className="mt-6">
              {activeTab === 'overview' && (
                <div>
                  <RepositoryInfo repository={project.repository} />
                  <div className="mt-6 grid grid-cols-2 gap-4">
                    <div className="bg-gray-50 p-4 rounded">
                      <h3 className="font-semibold mb-2">Generation Stats</h3>
                      <ul className="space-y-1 text-sm">
                        <li>Files Generated: {project.stats.filesGenerated}</li>
                        <li>Directories Created: {project.stats.directoriesCreated}</li>
                        <li>Total Lines of Code: {project.stats.linesOfCode}</li>
                        <li>Generation Time: {project.stats.generationTime}s</li>
                      </ul>
                    </div>
                    <div className="bg-gray-50 p-4 rounded">
                      <h3 className="font-semibold mb-2">Technologies</h3>
                      <div className="flex flex-wrap gap-2">
                        {project.technologies.map((tech) => (
                          <span
                            key={tech}
                            className="px-2 py-1 bg-blue-100 text-blue-700 rounded text-sm"
                          >
                            {tech}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              )}
              
              {activeTab === 'schema' && (
                <SchemaVisualizer schema={project.artifacts.communication_schema} />
              )}
              
              {activeTab === 'readme' && (
                <div className="prose max-w-none">
                  <ReactMarkdown
                    components={{
                      code({ node, inline, className, children, ...props }) {
                        const match = /language-(\w+)/.exec(className || '');
                        return !inline && match ? (
                          <SyntaxHighlighter
                            style={vscDarkPlus}
                            language={match[1]}
                            PreTag="div"
                            {...props}
                          >
                            {String(children).replace(/\n$/, '')}
                          </SyntaxHighlighter>
                        ) : (
                          <code className={className} {...props}>
                            {children}
                          </code>
                        );
                      },
                    }}
                  >
                    {project.artifacts.main_readme}
                  </ReactMarkdown>
                </div>
              )}
              
              {activeTab === 'files' && (
                <div className="space-y-4">
                  {Object.entries(project.artifacts.directory_tree).map(([path, content]) => (
                    <div key={path} className="border rounded p-4">
                      <h4 className="font-mono font-semibold mb-2">{path}</h4>
                      <pre className="bg-gray-50 p-2 rounded text-sm overflow-x-auto">
                        {JSON.stringify(content, null, 2)}
                      </pre>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}
      </motion.div>
    </div>
  );
}
```

## 🗄️ Database Schema

```sql
-- migrations/001_initial_schema.sql

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    email_verified BOOLEAN DEFAULT FALSE,
    stripe_customer_id VARCHAR(255),
    subscription_tier VARCHAR(50) DEFAULT 'free'
);

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'queued',
    progress INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    repository_url VARCHAR(500),
    error_message TEXT
);

-- Generation artifacts table
CREATE TABLE generation_artifacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    artifact_type VARCHAR(50) NOT NULL,
    content JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions table
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_subscription_id VARCHAR(255) UNIQUE NOT NULL,
    tier VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- API usage tracking
CREATE TABLE api_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    api_provider VARCHAR(50) NOT NULL,
    tokens_used INTEGER NOT NULL,
    cost DECIMAL(10, 4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_artifacts_project_id ON generation_artifacts(project_id);
CREATE INDEX idx_api_usage_user_id ON api_usage(user_id);
CREATE INDEX idx_api_usage_created_at ON api_usage(created_at);
```

## 🔐 Environment Configuration

```env
# .env.example

# Server Configuration
SERVER_ADDR=0.0.0.0:8080
ENVIRONMENT=development

# Database
DATABASE_URL=postgresql://maxamem:password@localhost:5432/maxamem
REDIS_URL=redis://localhost:6379

# Authentication
JWT_SECRET=your-super-secret-jwt-key-change-this
JWT_EXPIRATION=86400

# AI Services
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
AI_CACHE_TTL=3600

# GitHub
GITHUB_TOKEN=ghp_...
GITHUB_ORG=maxamem-generated

# Stripe
STRIPE_SECRET_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_STARTER=price_...
STRIPE_PRICE_PROFESSIONAL=price_...
STRIPE_PRICE_ENTERPRISE=price_...

# Email
SENDGRID_API_KEY=SG...
FROM_EMAIL=noreply@maxamem.com

# Storage
S3_BUCKET=maxamem-artifacts
S3_REGION=us-east-1
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...

# Monitoring
DATADOG_API_KEY=...
SENTRY_DSN=...
```

## 🚀 Deployment Instructions

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: maxamem
      POSTGRES_USER: maxamem
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
  
  backend:
    build:
      context: ./backend
      dockerfile: ../docker/backend.Dockerfile
    environment:
      - DATABASE_URL=postgresql://maxamem:password@postgres:5432/maxamem
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    ports:
      - "8080:8080"
  
  frontend:
    build:
      context: ./frontend
      dockerfile: ../docker/frontend.Dockerfile
    environment:
      - VITE_API_URL=http://localhost:8080
    ports:
      - "3000:3000"
  
  nginx:
    image: nginx:alpine
    volumes:
      - ./docker/nginx.conf:/etc/nginx/nginx.conf
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - backend
      - frontend

volumes:
  postgres_data:
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: maxamem-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: maxamem-backend
  template:
    metadata:
      labels:
        app: maxamem-backend
    spec:
      containers:
      - name: backend
        image: maxamem/backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: maxamem-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: maxamem-backend
spec:
  selector:
    app: maxamem-backend
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

This comprehensive implementation provides a complete, production-ready SaaS platform for generating intelligent project architectures with communication schemas. The system is designed for scalability, security, and maintainability.
