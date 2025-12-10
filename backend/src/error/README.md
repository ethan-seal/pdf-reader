# Error Handling

This module provides structured error handling for the PDF Reader API.

## Overview

The API uses a custom `ApiError` enum that automatically converts to proper HTTP responses with JSON error payloads. All errors are logged before being returned to the client.

## Error Types

### Client Errors (4xx)

- **`BadRequest`** - Invalid request data, malformed multipart uploads, etc.
  - HTTP Status: `400 Bad Request`
  - Use when: Client sent invalid or malformed data

- **`NotFound`** - Resource not found
  - HTTP Status: `404 Not Found`
  - Use when: Document ID doesn't exist, file not in storage

### Server Errors (5xx)

- **`InternalError`** - General server errors
  - HTTP Status: `500 Internal Server Error`
  - Use when: Unexpected errors, panics caught, unknown failures

- **`DatabaseError`** - Database operation failures
  - HTTP Status: `500 Internal Server Error`
  - Use when: SQL errors, connection issues, constraint violations

- **`StorageError`** - File storage failures
  - HTTP Status: `500 Internal Server Error`
  - Use when: Disk I/O errors, permission issues, storage full

- **`ExternalApiError`** - External API failures
  - HTTP Status: `502 Bad Gateway`
  - Use when: Claude API errors, third-party service failures

## Response Format

All errors return JSON with this structure:

```json
{
  "error": "ERROR_TYPE",
  "message": "Human-readable error message",
  "details": "Optional additional context"
}
```

### Examples

**Document not found:**
```json
{
  "error": "NOT_FOUND",
  "message": "Document not found: abc123"
}
```

**Database error:**
```json
{
  "error": "DATABASE_ERROR",
  "message": "Failed to save message: connection timeout"
}
```

**Invalid upload:**
```json
{
  "error": "BAD_REQUEST",
  "message": "Invalid multipart data: missing boundary"
}
```

## Usage in Handlers

### Basic Usage

```rust
use crate::error::ApiError;

pub async fn my_handler() -> Result<Json<Response>, ApiError> {
    // Return specific error types
    let data = fetch_data()
        .await
        .map_err(|e| ApiError::NotFound(format!("Resource not found: {}", e)))?;

    Ok(Json(data))
}
```

### Error Conversion

The module provides automatic conversions from common error types:

```rust
// From anyhow::Error
let result = some_operation().await?; // Automatically converts to ApiError

// From std::io::Error
let file = tokio::fs::read("file.txt").await?; // Automatically handles NotFound
```

### Choosing the Right Error Type

1. **Client mistakes** → `BadRequest` or `NotFound`
2. **Database problems** → `DatabaseError`
3. **Storage problems** → `StorageError`
4. **External API issues** → `ExternalApiError`
5. **Everything else** → `InternalError`

## Logging

All errors are automatically logged before being returned:

- **Client errors** (4xx) → Logged as `[WARN]`
- **Server errors** (5xx) → Logged as `[ERROR]`

This ensures all problems are captured in logs without manual logging in each handler.

## Benefits

1. **Consistent error format** - All endpoints return the same JSON structure
2. **Automatic logging** - Errors are logged at appropriate levels
3. **Type safety** - Compile-time guarantees about error handling
4. **Client-friendly** - Structured errors are easy to parse and handle
5. **Clean code** - Reduced boilerplate in handlers
6. **Better debugging** - Clear error types and messages
