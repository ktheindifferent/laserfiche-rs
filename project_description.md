# Laserfiche-RS Project Description

## Overview
Laserfiche-RS is a comprehensive Rust client library for interacting with the Laserfiche Repository API v1. It provides both async and blocking implementations for seamless integration with Laserfiche Cloud and self-hosted Laserfiche Server installations.

## Purpose
This library enables Rust applications to:
- Authenticate with Laserfiche repositories
- Manage documents and folders
- Handle metadata and field operations
- Perform search operations with OData support
- Manage templates, tags, and links
- Import and export documents with automatic MIME type detection

## Key Features
- **Dual Implementation**: Both async (default) and blocking API support
- **Type Safety**: Strongly typed request/response models
- **Comprehensive Coverage**: Full implementation of Laserfiche Repository API v1
- **Error Handling**: Robust error handling with detailed error types
- **Security**: OAuth 2.0 authentication with token refresh capabilities
- **File Support**: Automatic MIME type detection for common file formats

## Architecture
The codebase is organized into:
- `src/laserfiche.rs`: Main async implementation
- `src/laserfiche/blocking.rs`: Blocking API wrapper
- `src/main.rs`: Example usage and test functions
- `src/lib.rs`: Library entry point

## Recent Improvements (v0.0.7)
- Enhanced error handling by replacing unsafe `.unwrap()` calls
- Added automatic MIME type detection for file imports
- Implemented environment variable support for credentials
- Improved code organization and reduced duplication
- Updated documentation with better examples
- Fixed compilation warnings and added proper error propagation

## Usage
The library can be used in both async and blocking contexts, making it suitable for various Rust application architectures. Configuration is handled through environment variables for security, with fallback defaults for development.

## Target Audience
- Rust developers building document management systems
- Enterprise applications requiring Laserfiche integration
- Automation tools for document processing workflows
- Cloud and on-premise deployment scenarios