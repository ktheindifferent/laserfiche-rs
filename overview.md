# Laserfiche-RS Technical Overview

## Project Structure

```
laserfiche-rs/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Example usage and tests
│   ├── laserfiche.rs       # Main async API implementation
│   └── laserfiche/
│       └── blocking.rs     # Blocking API wrapper
├── Cargo.toml              # Project dependencies and metadata
├── README.md               # User documentation
├── LICENSE.md              # GPLv3 license
├── project_description.md  # Project summary
├── overview.md            # This file
└── todo.md                # Development tasks

```

## Core Components

### 1. Authentication Module
- **Location**: `src/laserfiche.rs` (lines 68-125)
- **Purpose**: Handle OAuth 2.0 authentication with Laserfiche
- **Key Features**:
  - Token generation and refresh
  - Secure credential handling
  - Timestamp tracking for token expiry

### 2. Entry Management
- **Location**: `src/laserfiche.rs` (lines 355-803)
- **Key Methods**:
  - `get()`: Retrieve entry information
  - `list()`: List folder contents
  - `search()`: OData-powered search
  - `import()`: Upload documents with MIME detection
  - `export()`: Download documents
  - `delete()`: Remove entries with audit trail
  - `patch()`: Move/rename operations
  - `copy()`: Duplicate entries

### 3. Metadata Operations
- **Location**: `src/laserfiche.rs` (lines 466-524)
- **Features**:
  - Get/update field values
  - Handle complex metadata structures
  - Support for multi-value fields

### 4. Template Management
- **Location**: `src/laserfiche.rs` (lines 904-1014)
- **Operations**:
  - Get assigned templates
  - Set/remove template associations
  - Template field management

### 5. Blocking API
- **Location**: `src/laserfiche/blocking.rs`
- **Purpose**: Provide synchronous alternatives for all async operations
- **Implementation**: Wraps async methods with blocking runtime

## Data Flow

```
Application
    ↓
[Environment Variables / Config]
    ↓
Authentication (OAuth 2.0)
    ↓
API Client (async/blocking)
    ↓
HTTP Requests (reqwest)
    ↓
Laserfiche API
    ↓
Response Parsing (serde)
    ↓
Type-safe Results
    ↓
Application
```

## Key Design Patterns

### 1. Result Enums
Each API operation returns a specialized enum (e.g., `EntryOrError`) that encapsulates either success data or error information, providing clear error handling paths.

### 2. Builder Pattern
API server configuration uses a struct-based approach for clean initialization.

### 3. Helper Functions
Common operations like URL building and MIME type detection are extracted into reusable functions.

### 4. Clone Optimization
While some cloning remains for API simplicity, critical paths have been optimized to reduce unnecessary allocations.

## Dependencies

### Core Dependencies
- `tokio`: Async runtime (v1.35+)
- `reqwest`: HTTP client with TLS support
- `serde`: Serialization/deserialization
- `serde_json`: JSON handling
- `error-chain`: Error management

### Security Dependencies
- `trust-dns-resolver`: Secure DNS resolution
- `native-tls`: Platform TLS implementation
- `openssl`: Cryptographic operations

## Security Considerations

1. **Credentials**: Never hardcoded, use environment variables
2. **Token Storage**: Auth tokens stored in memory with timestamps
3. **HTTPS Only**: All API communications use TLS
4. **Audit Trail**: Delete operations require audit comments

## Performance Optimizations

1. **Async by Default**: Non-blocking I/O for better concurrency
2. **Lazy Evaluation**: Operations only execute when awaited
3. **Efficient Serialization**: Direct JSON streaming where possible
4. **Connection Reuse**: HTTP client can maintain persistent connections

## Testing Strategy

The main.rs file includes example test functions for:
- Authentication and token refresh
- File import/export operations
- Entry listing and metadata retrieval
- Error handling scenarios

**Current State**: No formal unit tests exist yet. The project needs a comprehensive test suite with:
- Unit tests for all public API methods
- Integration tests with mock server
- Property-based testing for input validation
- Error scenario coverage

## Future Enhancements

Potential areas for improvement:
1. Connection pooling optimization
2. Retry logic with exponential backoff
3. Batch operation support
4. WebSocket support for real-time updates
5. Caching layer for frequently accessed data
6. Comprehensive integration test suite