# MaxaMem SaaS

AI-powered project documentation and GitHub scaffolding platform.

## Quick Start

1. **Setup environment:**
   ```bash
   make setup
   ```

2. **Configure API keys:**
   Edit `.env` file with your API keys

3. **Start development:**
   ```bash
   make dev
   ```

4. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080

## Features

- ✨ AI-powered documentation generation
- 📁 Complete project scaffolding
- 🔄 GitHub repository creation
- 📊 Interactive schema visualization
- 💳 Stripe subscription management
- 🔐 JWT authentication

## Tech Stack

- **Backend:** Rust (Actix-web)
- **Frontend:** React + TypeScript
- **Database:** PostgreSQL
- **Cache:** Redis
- **AI:** OpenAI + Claude

## Development

Run individual services:
```bash
# Backend only
cd backend && cargo run

# Frontend only
cd frontend && pnpm dev
```

## License

Proprietary - All rights reserved
