.PHONY: help setup dev build test clean docker-up docker-down

help:
	@echo "Available commands:"
	@echo "  make setup       - Initial setup"
	@echo "  make dev         - Start development environment"
	@echo "  make build       - Build for production"
	@echo "  make test        - Run tests"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make docker-up   - Start Docker services"
	@echo "  make docker-down - Stop Docker services"

setup: docker-up
	@echo "Setting up development environment..."
	@sleep 5
	cd backend && cargo build || true
	cd frontend && pnpm install

dev:
	@echo "Starting development servers..."
	make docker-up
	@echo "Starting backend on http://localhost:8080"
	cd backend && cargo run &
	@echo "Starting frontend on http://localhost:3000"
	cd frontend && pnpm dev

docker-up:
	@echo "Starting Docker services..."
	@if command -v docker-compose &> /dev/null; then \
		docker-compose up -d postgres redis; \
	else \
		echo "Docker Compose not available, skipping..."; \
	fi

docker-down:
	@echo "Stopping Docker services..."
	@if command -v docker-compose &> /dev/null; then \
		docker-compose down; \
	else \
		echo "Docker Compose not available, skipping..."; \
	fi

build:
	@echo "Building for production..."
	cd backend && cargo build --release
	cd frontend && pnpm build

test:
	@echo "Running tests..."
	cd backend && cargo test
	@echo "Frontend tests would run here"

clean:
	@echo "Cleaning build artifacts..."
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules
