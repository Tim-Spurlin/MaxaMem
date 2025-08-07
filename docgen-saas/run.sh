#!/bin/bash
echo "Starting DocGen SaaS..."

# Start Docker services if available
if command -v docker-compose &> /dev/null; then
    echo "Starting Docker services..."
    docker-compose up -d postgres redis
    sleep 5
fi

# Start backend
echo "Starting backend server..."
cd backend && cargo run &
BACKEND_PID=$!

# Start frontend
echo "Starting frontend..."
cd ../frontend && pnpm dev &
FRONTEND_PID=$!

echo ""
echo "Services running:"
echo "  Frontend: http://localhost:3000"
echo "  Backend:  http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for Ctrl+C
trap "kill $BACKEND_PID $FRONTEND_PID; docker-compose down; exit" INT
wait
