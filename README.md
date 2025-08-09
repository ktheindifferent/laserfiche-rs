# laserfiche-rs

A comprehensive Rust client library for the Laserfiche Repository API v1, supporting both Laserfiche Cloud and self-hosted Laserfiche Server installations.

[![Crates.io](https://img.shields.io/crates/v/laserfiche-rs.svg)](https://crates.io/crates/laserfiche-rs)
[![Documentation](https://docs.rs/laserfiche-rs/badge.svg)](https://docs.rs/laserfiche-rs)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Features

- **Full API Coverage**: Comprehensive implementation of Laserfiche Repository API v1 endpoints
- **Async and Blocking**: Both async (default) and blocking implementations available
- **Type Safety**: Strongly typed request and response models
- **Error Handling**: Robust error handling with detailed error types
- **Authentication**: OAuth 2.0 authentication support with token refresh capabilities
- **OData Support**: Search functionality with OData query parameters

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
laserfiche-rs = "0.0.6"
```

For blocking (synchronous) API usage:

```toml
[dependencies]
laserfiche-rs = { version = "0.0.6", features = ["blocking"] }
```

## Quick Start

### Environment Variables

For security, it's recommended to use environment variables for credentials:

```bash
export LF_API_ADDRESS="your-server.laserfiche.com"
export LF_REPOSITORY="your-repository"
export LF_USERNAME="your-username"
export LF_PASSWORD="your-password"
```

### Authentication

```rust
use laserfiche::{LFApiServer, Auth, AuthOrError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let api_server = LFApiServer {
        address: env::var("LF_API_ADDRESS")
            .unwrap_or_else(|_| "your-server.laserfiche.com".to_string()),
        repository: env::var("LF_REPOSITORY")
            .unwrap_or_else(|_| "your-repository".to_string()),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        env::var("LF_USERNAME")?,
        env::var("LF_PASSWORD")?
    ).await?;

    match auth_result {
        AuthOrError::Auth(auth) => {
            println!("Authenticated successfully!");
            // Use auth token for API calls
        },
        AuthOrError::LFAPIError(error) => {
            eprintln!("Authentication failed: {:?}", error);
            return Err(format!("Authentication failed: {:?}", error).into());
        }
    }
    
    Ok(())
}
```

### Entry Management

```rust
use laserfiche::{Entry, EntryOrError};

// Get entry information
let entry_result = Entry::get(api_server.clone(), auth.clone(), entry_id).await?;

match entry_result {
    EntryOrError::Entry(entry) => {
        println!("Entry name: {}", entry.name);
        println!("Entry type: {}", entry.entry_type);
        println!("Full path: {}", entry.full_path);
    },
    EntryOrError::LFAPIError(error) => {
        eprintln!("Failed to get entry: {:?}", error);
    }
}

// List folder contents
let entries_result = Entry::list(api_server.clone(), auth.clone(), folder_id).await?;

match entries_result {
    EntriesOrError::Entries(entries) => {
        for entry in entries.value {
            println!("- {} ({})", entry.name, entry.entry_type);
        }
    },
    EntriesOrError::LFAPIError(error) => {
        eprintln!("Failed to list entries: {:?}", error);
    }
}
```

### Document Operations

```rust
// Import a document - MIME type is automatically detected from file extension
let import_result = Entry::import(
    api_server.clone(),
    auth.clone(),
    "/path/to/file.pdf".to_string(),  // Supports PDF, PNG, JPG, DOCX, etc.
    "document_name.pdf".to_string(),
    parent_folder_id
).await?;

// Export a document
let export_result = Entry::export(
    api_server.clone(),
    auth.clone(),
    document_id,
    "/path/to/save/file.pdf"
).await?;

// Copy an entry
let copy_result = Entry::copy(
    api_server.clone(),
    auth.clone(),
    source_entry_id,
    target_folder_id,
    Some("new_name".to_string())
).await?;
```

### Search with OData

```rust
// Search for entries with OData parameters
let search_result = Entry::search(
    api_server.clone(),
    auth.clone(),
    "invoice".to_string(),              // search query
    Some("name asc".to_string()),       // order by name ascending
    Some("id,name,entryType".to_string()), // select specific fields
    Some(0),                             // skip first N results
    Some(50)                             // return max 50 results
).await?;

match search_result {
    EntriesOrError::Entries(entries) => {
        println!("Found {} entries", entries.value.len());
        for entry in entries.value {
            println!("- {}: {}", entry.id, entry.name);
        }
    },
    EntriesOrError::LFAPIError(error) => {
        eprintln!("Search failed: {:?}", error);
    }
}
```

### Metadata/Field Operations

```rust
use serde_json::json;

// Get metadata fields
let metadata_result = Entry::get_metadata(
    api_server.clone(),
    auth.clone(),
    entry_id
).await?;

// Update metadata fields
let metadata_update = json!({
    "value": [
        {
            "fieldName": "Invoice Number",
            "values": [{"value": "INV-2024-001"}]
        },
        {
            "fieldName": "Amount",
            "values": [{"value": "1500.00"}]
        }
    ]
});

let update_result = Entry::update_metadata(
    api_server.clone(),
    auth.clone(),
    entry_id,
    metadata_update
).await?;
```

### Template Management

```rust
// Get template assigned to an entry
let template_result = Entry::get_template(
    api_server.clone(),
    auth.clone(),
    entry_id
).await?;

// Assign a template to an entry
let assign_result = Entry::set_template(
    api_server.clone(),
    auth.clone(),
    entry_id,
    "Invoice Template".to_string()
).await?;

// Remove template from an entry
let remove_result = Entry::remove_template(
    api_server.clone(),
    auth.clone(),
    entry_id
).await?;
```

### Tag Management

```rust
// Get tags assigned to an entry
let tags_result = Entry::get_tags(
    api_server.clone(),
    auth.clone(),
    entry_id
).await?;

// Assign tags to an entry
let tag_ids = vec![101, 102, 103];
let assign_tags_result = Entry::set_tags(
    api_server.clone(),
    auth.clone(),
    entry_id,
    tag_ids
).await?;
```

### Folder Operations

```rust
// Create a new folder
let new_folder_result = Entry::new_path(
    api_server.clone(),
    auth.clone(),
    "New Folder Name".to_string(),
    "Default".to_string(),  // volume name
    parent_folder_id
).await?;

// Move or rename an entry
let move_result = Entry::patch(
    api_server.clone(),
    auth.clone(),
    entry_id,
    Some(new_parent_folder_id),  // new parent (for move)
    Some("New Name".to_string())  // new name (for rename)
).await?;

// Delete an entry
let delete_result = Entry::delete(
    api_server.clone(),
    auth.clone(),
    entry_id,
    "Deletion reason for audit".to_string()
).await?;
```

### Links Management

```rust
// Get links associated with an entry
let links_result = Entry::get_links(
    api_server.clone(),
    auth.clone(),
    entry_id
).await?;

match links_result {
    LinksOrError::Links(links) => {
        for link in links.value {
            println!("Link: {} -> {} ({})", link.source_id, link.target_id, link.link_type);
        }
    },
    LinksOrError::LFAPIError(error) => {
        eprintln!("Failed to get links: {:?}", error);
    }
}
```

## Blocking API

For synchronous/blocking operations, use the `blocking` module:

```rust
use laserfiche::blocking::{Auth, Entry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_server = laserfiche::blocking::LFApiServer {
        address: "your-server.laserfiche.com".to_string(),
        repository: "your-repository".to_string(),
    };

    // Blocking authentication
    let auth_result = Auth::new(
        api_server.clone(),
        "username".to_string(),
        "password".to_string()
    )?;

    // Blocking API calls
    let entry_result = Entry::get(api_server.clone(), auth, entry_id)?;
    
    Ok(())
}
```

## Supported File Types

The library automatically detects MIME types for common file extensions:
- Documents: PDF, DOC, DOCX, TXT, XML, JSON
- Images: PNG, JPG/JPEG, GIF, TIFF/TIF
- Spreadsheets: XLS, XLSX
- Other formats default to `application/octet-stream`

## API Methods Reference

### Authentication
- `Auth::new()` - Authenticate with username/password
- `Auth::refresh()` - Refresh authentication token

### Entry Operations
- `Entry::get()` - Get entry by ID
- `Entry::list()` - List folder contents
- `Entry::search()` - Search entries with OData support
- `Entry::delete()` - Delete an entry
- `Entry::patch()` - Move or rename an entry
- `Entry::copy()` - Copy an entry to a new location

### Document Operations
- `Entry::import()` - Import a document
- `Entry::export()` - Export/download a document
- `Entry::edoc_head()` - Get document headers

### Metadata/Fields
- `Entry::get_metadata()` - Get entry metadata
- `Entry::update_metadata()` - Update entry metadata
- `Entry::get_fields()` - Get all fields
- `Entry::get_field()` - Get specific field

### Template Management
- `Entry::get_template()` - Get assigned template
- `Entry::set_template()` - Assign template
- `Entry::remove_template()` - Remove template

### Tag Management
- `Entry::get_tags()` - Get assigned tags
- `Entry::set_tags()` - Assign tags

### Link Management
- `Entry::get_links()` - Get entry links

### Folder Operations
- `Entry::new_path()` - Create new folder

## Error Handling

All API methods return a `Result` type with specific error enums for each operation:

- `AuthOrError` - Authentication results
- `EntryOrError` - Single entry operations
- `EntriesOrError` - Multiple entries operations
- `MetadataResultOrError` - Metadata operations
- `ImportResultOrError` - Import operations
- `BitsOrError` - Export operations
- `TemplateOrError` - Template operations
- `TagsOrError` - Tag operations
- `LinksOrError` - Link operations

Each error enum contains either the successful result or an `LFAPIError` with detailed error information.

### Example Error Handling

```rust
use laserfiche::{Entry, EntryOrError};

let entry_result = Entry::get(api_server.clone(), auth.clone(), entry_id).await?;

match entry_result {
    EntryOrError::Entry(entry) => {
        println!("Entry retrieved: {}", entry.name);
        // Process the entry
    },
    EntryOrError::LFAPIError(error) => {
        eprintln!("API Error: {:?}", error);
        // Handle the error appropriately
        return Err(format!("Failed to get entry: {:?}", error).into());
    }
}
```

## Configuration

### Self-Hosted vs Cloud

For self-hosted Laserfiche Server:
```rust
let api_server = LFApiServer {
    address: "your-server.example.com".to_string(),
    repository: "your-repository".to_string(),
};
```

For Laserfiche Cloud:
```rust
let api_server = LFApiServer {
    address: "api.laserfiche.com".to_string(),  // or api.eu.laserfiche.com for EU
    repository: "your-repository-id".to_string(),
};
```

## Development

### Building from Source

```bash
git clone https://github.com/PixelCoda/laserfiche-rs.git
cd laserfiche-rs
cargo build --release
```

### Running Tests

```bash
# Set up environment variables first
export LF_API_ADDRESS="your-test-server.laserfiche.com"
export LF_REPOSITORY="test-repository"
export LF_USERNAME="test-username"
export LF_PASSWORD="test-password"

# Run tests
cargo test

# Run example
cargo run --example basic_usage
```

## License

Licensed under GPLv3. See [LICENSE](LICENSE.md) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Guidelines

1. Ensure all tests pass before submitting
2. Add tests for new functionality
3. Update documentation as needed
4. Follow Rust standard formatting (`cargo fmt`)
5. Ensure no clippy warnings (`cargo clippy`)

## Support

For issues and questions, please use the [GitHub issue tracker](https://github.com/PixelCoda/laserfiche-rs/issues).

## Changelog

### v0.0.7 (Upcoming)
- Improved error handling with better `.unwrap()` safety
- Automatic MIME type detection for file imports
- Environment variable support for credentials
- Code cleanup and documentation improvements
- Fixed hardcoded values and improved configurability

### v0.0.6
- Initial stable release with full API coverage