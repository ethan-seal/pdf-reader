# Claude Development Guidelines

This document contains guidelines for AI assistants (like Claude) working on this project.

## Running the Application

### DO NOT run background tasks for build/dev servers

**Important:** Do not automatically start background processes for building or running the backend/frontend without explicit user request.

Reasons:
- The user may already have dev servers running
- Multiple instances can cause port conflicts
- Background processes can accumulate and waste resources
- Users prefer to control when services start/stop

### If the user asks to start services:

**Backend (Rust):**
```bash
cd backend
ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY} cargo run
```

**Frontend (SolidJS):**
```bash
cd frontend
npm run dev
```

## Development Workflow

- Always read files before modifying them
- Use the Edit tool for existing files, not Write
- Test builds with `npm run build` (frontend) or `cargo build` (backend)
- Only create commits when explicitly requested by the user

## Project Structure

- `backend/` - Rust/Axum API server
- `frontend/` - SolidJS frontend application
- Backend runs on port 3001
- Frontend runs on port 3000
