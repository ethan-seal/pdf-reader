# Backend Dockerfile

This Dockerfile builds the Rust backend service for the broadband map project.

## Features

- **Multi-stage build**: Separates build and runtime stages to minimize final image size
- **Dependency caching**: Dependencies are built in a separate layer for faster rebuilds
- **Security**: Runs as non-root user (appuser)
- **Optimized runtime**: Uses debian:bookworm-slim with only necessary runtime dependencies

## Build

```bash
docker build -t broadband-map-backend .
```

## Run

```bash
docker run -p 3001:3001 \
  -e ANTHROPIC_API_KEY=your_api_key \
  -v $(pwd)/uploads:/app/uploads \
  broadband-map-backend
```

## Environment Variables

- `ANTHROPIC_API_KEY` (required): API key for Claude integration

## Ports

- `3001`: HTTP API server

## Volumes

- `/app/uploads`: Directory for uploaded documents (optional, for persistence)

## Build Arguments

The Dockerfile uses Rust 1.75 and Debian Bookworm as base images.
