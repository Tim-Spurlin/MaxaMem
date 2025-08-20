.PHONY: help dev build start test lint clean setup install-deps

# Default target
help:
	@echo "MaxaMem Development Commands"
	@echo "=========================="
	@echo "setup       - Initial project setup"
	@echo "install-deps- Install all dependencies"
	@echo "dev         - Start development servers"
	@echo "build       - Build the project"
	@echo "start       - Start production servers"
	@echo "test        - Run tests"
	@echo "lint        - Run linters"
	@echo "clean       - Clean build artifacts"

# Initial setup
setup: install-deps
	@echo "Setting up MaxaMem development environment..."
	@echo "Please ensure PostgreSQL and Redis are running"
	@echo "Update backend/.env with your database configuration"

# Install dependencies
install-deps:
	@echo "Installing backend dependencies..."
	cd backend && cargo check
	@echo "Installing frontend dependencies..."
	cd frontend && npm install

# Development mode
dev:
	@echo "Starting MaxaMem in development mode..."
	@echo "Backend will run on http://localhost:8000"
	@echo "Frontend will run on http://localhost:3000"
	cd backend && cargo run &
	cd frontend && npm run dev

# Production build
build:
	@echo "Building MaxaMem for production..."
	cd backend && cargo build --release
	cd frontend && npm run build

# Start production
start:
	@echo "Starting MaxaMem in production mode..."
	cd backend && ./target/release/maxamem-backend &
	cd frontend && npm run preview

# Run tests
test:
	@echo "Running tests..."
	cd backend && cargo test
	cd frontend && npm test

# Run linters
lint:
	@echo "Running linters..."
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run lint

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules/.vite