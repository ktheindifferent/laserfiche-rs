# Laserfiche-RS Codebase Documentation

## Project Overview

**laserfiche-rs** is a comprehensive Rust client library for the Laserfiche Repository API v1. It provides robust integration capabilities for both Laserfiche Cloud and self-hosted Laserfiche Server installations.

### Key Characteristics
- **Language**: Rust (Edition 2021)
- **Version**: 0.0.6 (v0.0.7 in development)
- **License**: GPL-3.0-or-later
- **Author**: Caleb Mitchell Smith
- **Repository**: https://github.com/PixelCoda/laserfiche-rs
- **Documentation**: https://docs.rs/laserfiche-rs

## Architecture

### Core Design Principles
1. **Dual Implementation**: Both async (default) and blocking API support
2. **Type Safety**: Strongly typed request/response models with custom error enums
3. **Security First**: OAuth 2.0 authentication with environment variable support for credentials
4. **Comprehensive Coverage**: Full implementation of Laserfiche Repository API v1 endpoints

### Project Structure

```
laserfiche-rs/
├── src/
│   ├── lib.rs                 # Library entry point and public API exports
│   ├── main.rs                # Example usage and test functions
│   ├── laserfiche.rs          # Main async API implementation (~1000+ lines)
│   └── laserfiche/
│       └── blocking.rs        # Blocking API wrapper for sync operations
├── tests/
│   └── integration_tests.rs  # Integration test suite (15+ tests)
├── Cargo.toml                 # Dependencies and package metadata
├── README.md                  # User-facing documentation
├── LICENSE.md                 # GPLv3 license
├── project_description.md     # High-level project summary
├── overview.md               # Technical architecture overview
├── todo.md                   # Development task tracking
└── api.paw                   # API testing collection
```

## Core Components

### 1. Authentication System (`Auth`)
- **Location**: `src/laserfiche.rs` (lines 68-125)
- **Features**:
  - OAuth 2.0 token generation
  - Automatic token refresh capabilities
  - Secure credential handling via environment variables
  - Timestamp tracking for token expiry

### 2. Entry Management (`Entry`)
- **Location**: `src/laserfiche.rs` (lines 355-803)
- **Key Operations**:
  - `get()` - Retrieve entry by ID
  - `list()` - List folder contents
  - `search()` - OData-powered search with filtering
  - `import()` - Upload documents with automatic MIME detection
  - `export()` - Download documents
  - `delete()` - Remove entries with audit trail
  - `patch()` - Move/rename operations
  - `copy()` - Duplicate entries
  - `new_path()` - Create new folders

### 3. Metadata Operations
- **Location**: `src/laserfiche.rs` (lines 466-524)
- **Capabilities**:
  - Get/update field values
  - Handle complex metadata structures
  - Support for multi-value fields
  - Field-level operations

### 4. Template Management
- **Location**: `src/laserfiche.rs` (lines 904-1014)
- **Functions**:
  - Get assigned templates
  - Set/remove template associations
  - Template field management

### 5. Tag & Link Management
- **Location**: `src/laserfiche.rs` (lines 1015-1100)
- **Features**:
  - Tag assignment and retrieval
  - Link relationship management
  - Batch tag operations

## Dependencies

### Core Dependencies
```toml
serde = "1.0" (with derive)
serde_json = "1.0"
tokio = "1.35.0" (full features)
reqwest = "0.11.9" (blocking, json, multipart)
error-chain = "0.12.4"
urlencoding = "2.1"
trust-dns-resolver = "0.20"
```

### Feature Flags
- `default`: Includes TLS support via native-tls
- `blocking`: Enables synchronous API wrapper

## API Coverage

### Implemented Endpoints
- ✅ Authentication (OAuth 2.0)
- ✅ Entry CRUD operations
- ✅ Document import/export
- ✅ Metadata/field management
- ✅ Template operations
- ✅ Tag management
- ✅ Link operations
- ✅ Search with OData
- ✅ Folder operations

### Response Handling
All API methods return specialized Result enums:
- `AuthOrError` - Authentication results
- `EntryOrError` - Single entry operations
- `EntriesOrError` - Multiple entries operations
- `MetadataResultOrError` - Metadata operations
- `ImportResultOrError` - Import operations
- `BitsOrError` - Export operations
- `TemplateOrError` - Template operations
- `TagsOrError` - Tag operations
- `LinksOrError` - Link operations

## Recent Improvements (v0.0.7)

### Code Quality
- ✅ Replaced unsafe `.unwrap()` calls with proper error propagation
- ✅ Added automatic MIME type detection for 15+ file formats
- ✅ Implemented environment variable support for secure credential management
- ✅ Reduced code duplication through helper functions
- ✅ Fixed compilation warnings and improved error handling

### Testing
- ✅ Added comprehensive unit test suite (15+ tests)
- ✅ Created integration tests for core API operations
- ✅ Implemented test coverage for authentication, import/export, and metadata operations

### Documentation
- ✅ Enhanced README with detailed examples
- ✅ Added inline documentation for public APIs
- ✅ Created architectural overview documentation
- ✅ Maintained development task tracking

## File Format Support

Automatic MIME type detection for:
- **Documents**: PDF, DOC, DOCX, TXT, XML, JSON
- **Images**: PNG, JPG/JPEG, GIF, TIFF/TIF
- **Spreadsheets**: XLS, XLSX, CSV
- **Presentations**: PPT, PPTX
- **Default**: application/octet-stream for unknown types

## Security Considerations

1. **Credentials**: Never hardcoded; use environment variables:
   - `LF_API_ADDRESS`
   - `LF_REPOSITORY`
   - `LF_USERNAME`
   - `LF_PASSWORD`

2. **Token Management**: Auth tokens stored in memory with expiry tracking
3. **HTTPS Only**: All API communications use TLS
4. **Audit Trail**: Delete operations require audit comments

## Testing Strategy

### Current Test Coverage
- **Unit Tests**: 15+ tests covering core structs and functions
- **Integration Tests**: Authentication, API operations, error handling
- **Test Files**: 
  - `tests/integration_tests.rs` - Main integration test suite
  - `src/main.rs` - Example usage and manual test functions

### Test Categories
1. Authentication and token refresh
2. Entry CRUD operations
3. Document import/export with MIME detection
4. Metadata operations
5. Error handling scenarios

## Development Workflow

### Building
```bash
cargo build --release
```

### Testing
```bash
# Set environment variables
export LF_API_ADDRESS="your-server.laserfiche.com"
export LF_REPOSITORY="your-repository"
export LF_USERNAME="your-username"
export LF_PASSWORD="your-password"

# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Code Quality
```bash
cargo fmt      # Format code
cargo clippy   # Lint code
```

## Known Issues & TODOs

### High Priority
- [ ] Implement retry logic with exponential backoff
- [ ] Add connection pooling for performance
- [ ] Create mock server for integration tests
- [ ] Add request/response logging (with data masking)
- [ ] Implement proper timeout configurations

### Medium Priority
- [ ] Add batch operation support
- [ ] Implement caching layer
- [ ] Add progress callbacks for large files
- [ ] Create builder pattern for complex requests
- [ ] Add WebSocket support for real-time updates

### Documentation Needs
- [ ] Comprehensive API guide
- [ ] Troubleshooting section
- [ ] Performance tuning guide
- [ ] Migration guide from other libraries

## Usage Patterns

### Async Pattern
```rust
use laserfiche::{LFApiServer, Auth, Entry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_server = LFApiServer { /* config */ };
    let auth = Auth::new(/* credentials */).await?;
    let entry = Entry::get(api_server, auth, entry_id).await?;
    Ok(())
}
```

### Blocking Pattern
```rust
use laserfiche::blocking::{LFApiServer, Auth, Entry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_server = LFApiServer { /* config */ };
    let auth = Auth::new(/* credentials */)?;
    let entry = Entry::get(api_server, auth, entry_id)?;
    Ok(())
}
```

## Performance Characteristics

- **Async by Default**: Non-blocking I/O for better concurrency
- **Lazy Evaluation**: Operations only execute when awaited
- **Efficient Serialization**: Direct JSON streaming where possible
- **Connection Reuse**: HTTP client maintains persistent connections

## Contributing Guidelines

1. Ensure all tests pass before submitting PRs
2. Add tests for new functionality
3. Update documentation as needed
4. Follow Rust standard formatting (`cargo fmt`)
5. Ensure no clippy warnings (`cargo clippy`)
6. Use environment variables for sensitive data

## Version History

- **v0.0.6**: Current stable release with full API coverage
- **v0.0.7** (in development): Enhanced error handling, MIME detection, environment variables, test suite

## Support

- **Issues**: https://github.com/PixelCoda/laserfiche-rs/issues
- **Documentation**: https://docs.rs/laserfiche-rs
- **Repository**: https://github.com/PixelCoda/laserfiche-rs

---

*Last Updated: 2025-08-15*
*This document is maintained to provide AI assistants and developers with comprehensive codebase context.*