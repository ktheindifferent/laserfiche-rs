// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.

use regex::Regex;
use std::path::PathBuf;
use error_chain::error_chain;
use once_cell::sync::Lazy;
use url::Url;

// Regular expressions for validation
static SQL_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER|EXEC|EXECUTE|UNION|--|;|'|\\x00|\\n|\\r|\\x1a)").unwrap()
});

static SCRIPT_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(<script|javascript:|on\w+\s*=|eval\(|alert\(|document\.|window\.)").unwrap()
});

static VALID_REPOSITORY_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9\-_]{0,63}$").unwrap()
});

static VALID_FIELD_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_\-\s]{0,127}$").unwrap()
});

error_chain! {
    errors {
        InvalidEntryId(id: i64) {
            description("Invalid entry ID")
            display("Invalid entry ID: {}. Entry IDs must be positive integers.", id)
        }
        InvalidFilePath(path: String) {
            description("Invalid file path")
            display("Invalid file path: {}. Path contains invalid characters or path traversal attempts.", path)
        }
        PathTraversalAttempt(path: String) {
            description("Path traversal attempt detected")
            display("Path traversal attempt detected in: {}", path)
        }
        InvalidRepositoryName(name: String) {
            description("Invalid repository name")
            display("Invalid repository name: {}. Repository names must be alphanumeric with hyphens or underscores, 1-64 characters.", name)
        }
        InvalidUrl(url: String) {
            description("Invalid URL")
            display("Invalid URL: {}", url)
        }
        InsecureUrl(url: String) {
            description("Insecure URL")
            display("Insecure URL: {}. HTTPS is required for API endpoints.", url)
        }
        InvalidFieldName(name: String) {
            description("Invalid field name")
            display("Invalid field name: {}. Field names must start with a letter and contain only alphanumeric characters, underscores, hyphens, or spaces.", name)
        }
        InvalidFieldValue(value: String) {
            description("Invalid field value")
            display("Invalid field value: contains potentially malicious content")
        }
        SqlInjectionAttempt(input: String) {
            description("SQL injection attempt detected")
            display("SQL injection pattern detected in input")
        }
        ScriptInjectionAttempt(input: String) {
            description("Script injection attempt detected")
            display("Script injection pattern detected in input")
        }
        FileSizeTooLarge(size: u64, max: u64) {
            description("File size exceeds maximum allowed")
            display("File size {} bytes exceeds maximum allowed size of {} bytes", size, max)
        }
        InvalidFileName(name: String) {
            description("Invalid file name")
            display("Invalid file name: {}", name)
        }
    }
}

/// Maximum file size for uploads (100MB)
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum field value length (10KB)
pub const MAX_FIELD_VALUE_LENGTH: usize = 10 * 1024;

/// Validate an entry ID
pub fn validate_entry_id(id: i64) -> Result<i64> {
    if id <= 0 {
        return Err(ErrorKind::InvalidEntryId(id).into());
    }
    // Check for reasonable upper bound (prevent overflow attacks)
    if id > i64::MAX / 2 {
        return Err(ErrorKind::InvalidEntryId(id).into());
    }
    Ok(id)
}

/// Validate and sanitize a file path
pub fn validate_file_path(path: &str) -> Result<PathBuf> {
    // Check for empty path
    if path.is_empty() {
        return Err(ErrorKind::InvalidFilePath(path.to_string()).into());
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(ErrorKind::InvalidFilePath(path.to_string()).into());
    }

    // Check for path traversal attempts
    if path.contains("..") || path.contains("~") {
        return Err(ErrorKind::PathTraversalAttempt(path.to_string()).into());
    }

    // Additional checks for Windows-specific path traversal
    if cfg!(windows) {
        if path.contains(r"..\" ) || path.contains(r"\..") {
            return Err(ErrorKind::PathTraversalAttempt(path.to_string()).into());
        }
    }

    let path_buf = PathBuf::from(path);
    
    // Canonicalize the path to resolve any symbolic links and ensure it's absolute
    // Note: This will fail if the path doesn't exist, which is what we want for imports
    match path_buf.canonicalize() {
        Ok(canonical_path) => {
            // Ensure the path doesn't escape to parent directories
            let path_str = canonical_path.to_string_lossy();
            if path_str.contains("..") {
                return Err(ErrorKind::PathTraversalAttempt(path.to_string()).into());
            }
            Ok(canonical_path)
        },
        Err(_) => {
            // For new files that don't exist yet, validate the parent directory
            if let Some(parent) = path_buf.parent() {
                if parent.exists() {
                    // Parent exists, path is likely valid for creation
                    Ok(path_buf)
                } else {
                    Err(ErrorKind::InvalidFilePath(path.to_string()).into())
                }
            } else {
                Err(ErrorKind::InvalidFilePath(path.to_string()).into())
            }
        }
    }
}

/// Validate a repository name
pub fn validate_repository_name(name: &str) -> Result<String> {
    // Check for empty name
    if name.is_empty() {
        return Err(ErrorKind::InvalidRepositoryName(name.to_string()).into());
    }

    // Check length
    if name.len() > 64 {
        return Err(ErrorKind::InvalidRepositoryName(name.to_string()).into());
    }

    // Check for SQL injection patterns
    if SQL_INJECTION_PATTERN.is_match(name) {
        return Err(ErrorKind::SqlInjectionAttempt(name.to_string()).into());
    }

    // Check format (alphanumeric with hyphens and underscores)
    if !VALID_REPOSITORY_NAME.is_match(name) {
        return Err(ErrorKind::InvalidRepositoryName(name.to_string()).into());
    }

    Ok(name.to_string())
}

/// Validate a URL for API server addresses
pub fn validate_api_url(url: &str) -> Result<String> {
    // Check for empty URL
    if url.is_empty() {
        return Err(ErrorKind::InvalidUrl(url.to_string()).into());
    }

    // Parse the URL
    let parsed_url = Url::parse(url)
        .map_err(|_| ErrorKind::InvalidUrl(url.to_string()))?;

    // Check for HTTPS (required for security)
    if parsed_url.scheme() != "https" {
        return Err(ErrorKind::InsecureUrl(url.to_string()).into());
    }

    // Check for valid host
    if parsed_url.host_str().is_none() {
        return Err(ErrorKind::InvalidUrl(url.to_string()).into());
    }

    // Check for SQL injection in URL
    if SQL_INJECTION_PATTERN.is_match(url) {
        return Err(ErrorKind::SqlInjectionAttempt(url.to_string()).into());
    }

    Ok(url.to_string())
}

/// Validate an API server address (hostname or FQDN)
pub fn validate_server_address(address: &str) -> Result<String> {
    // Check for empty address
    if address.is_empty() {
        return Err(ErrorKind::InvalidUrl(address.to_string()).into());
    }

    // Check length
    if address.len() > 253 {
        return Err(ErrorKind::InvalidUrl(address.to_string()).into());
    }

    // Check for SQL injection
    if SQL_INJECTION_PATTERN.is_match(address) {
        return Err(ErrorKind::SqlInjectionAttempt(address.to_string()).into());
    }

    // Basic validation for domain name format
    let domain_regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9\-\.]{0,251}[a-zA-Z0-9]$").unwrap();
    if !domain_regex.is_match(address) {
        return Err(ErrorKind::InvalidUrl(address.to_string()).into());
    }

    // Check each label in the domain
    for label in address.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(ErrorKind::InvalidUrl(address.to_string()).into());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ErrorKind::InvalidUrl(address.to_string()).into());
        }
    }

    Ok(address.to_string())
}

/// Validate a field name
pub fn validate_field_name(name: &str) -> Result<String> {
    // Check for empty name
    if name.is_empty() {
        return Err(ErrorKind::InvalidFieldName(name.to_string()).into());
    }

    // Check length
    if name.len() > 128 {
        return Err(ErrorKind::InvalidFieldName(name.to_string()).into());
    }

    // Check for injection patterns
    if SQL_INJECTION_PATTERN.is_match(name) {
        return Err(ErrorKind::SqlInjectionAttempt(name.to_string()).into());
    }

    if SCRIPT_INJECTION_PATTERN.is_match(name) {
        return Err(ErrorKind::ScriptInjectionAttempt(name.to_string()).into());
    }

    // Check format
    if !VALID_FIELD_NAME.is_match(name) {
        return Err(ErrorKind::InvalidFieldName(name.to_string()).into());
    }

    Ok(name.to_string())
}

/// Validate and sanitize a field value
pub fn validate_field_value(value: &str) -> Result<String> {
    // Check length
    if value.len() > MAX_FIELD_VALUE_LENGTH {
        return Err(ErrorKind::InvalidFieldValue(
            format!("Value exceeds maximum length of {} characters", MAX_FIELD_VALUE_LENGTH)
        ).into());
    }

    // Check for script injection
    if SCRIPT_INJECTION_PATTERN.is_match(value) {
        return Err(ErrorKind::ScriptInjectionAttempt(value.to_string()).into());
    }

    // Allow SQL-like patterns in values but escape them
    let sanitized = value
        .replace('\'', "''")  // Escape single quotes
        .replace('\\', "\\\\") // Escape backslashes
        .replace('\0', "")     // Remove null bytes
        .replace('\x1a', "");  // Remove SUB character

    Ok(sanitized)
}

/// Validate a file name
pub fn validate_file_name(name: &str) -> Result<String> {
    // Check for empty name
    if name.is_empty() {
        return Err(ErrorKind::InvalidFileName(name.to_string()).into());
    }

    // Check length
    if name.len() > 255 {
        return Err(ErrorKind::InvalidFileName(name.to_string()).into());
    }

    // Check for null bytes
    if name.contains('\0') {
        return Err(ErrorKind::InvalidFileName(name.to_string()).into());
    }

    // Check for path traversal in filename
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(ErrorKind::InvalidFileName(name.to_string()).into());
    }

    // Check for invalid characters (platform-specific)
    let invalid_chars = if cfg!(windows) {
        r#"<>:"|?*"#
    } else {
        ""
    };

    for ch in invalid_chars.chars() {
        if name.contains(ch) {
            return Err(ErrorKind::InvalidFileName(name.to_string()).into());
        }
    }

    // Check for reserved names on Windows
    if cfg!(windows) {
        let name_upper = name.to_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", 
                       "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", 
                       "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        
        for reserved_name in &reserved {
            if name_upper == *reserved_name || name_upper.starts_with(&format!("{}.", reserved_name)) {
                return Err(ErrorKind::InvalidFileName(name.to_string()).into());
            }
        }
    }

    Ok(name.to_string())
}

/// Validate file size
pub fn validate_file_size(size: u64) -> Result<u64> {
    if size > MAX_FILE_SIZE {
        return Err(ErrorKind::FileSizeTooLarge(size, MAX_FILE_SIZE).into());
    }
    Ok(size)
}

/// Validate JSON metadata object
pub fn validate_metadata_json(metadata: &serde_json::Value) -> Result<serde_json::Value> {
    match metadata {
        serde_json::Value::Object(map) => {
            let mut validated_map = serde_json::Map::new();
            
            for (key, value) in map {
                // Validate field name
                let validated_key = validate_field_name(key)?;
                
                // Validate field value based on type
                let validated_value = match value {
                    serde_json::Value::String(s) => {
                        serde_json::Value::String(validate_field_value(s)?)
                    },
                    serde_json::Value::Array(arr) => {
                        let mut validated_arr = Vec::new();
                        for item in arr {
                            if let serde_json::Value::String(s) = item {
                                validated_arr.push(serde_json::Value::String(validate_field_value(s)?));
                            } else {
                                validated_arr.push(item.clone());
                            }
                        }
                        serde_json::Value::Array(validated_arr)
                    },
                    _ => value.clone()
                };
                
                validated_map.insert(validated_key, validated_value);
            }
            
            Ok(serde_json::Value::Object(validated_map))
        },
        _ => Ok(metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_entry_id() {
        // Valid IDs
        assert!(validate_entry_id(1).is_ok());
        assert!(validate_entry_id(12345).is_ok());
        assert!(validate_entry_id(999999).is_ok());

        // Invalid IDs
        assert!(validate_entry_id(0).is_err());
        assert!(validate_entry_id(-1).is_err());
        assert!(validate_entry_id(-12345).is_err());
        assert!(validate_entry_id(i64::MAX).is_err());
    }

    #[test]
    fn test_validate_repository_name() {
        // Valid names
        assert!(validate_repository_name("my-repository").is_ok());
        assert!(validate_repository_name("test_repo_123").is_ok());
        assert!(validate_repository_name("Repository1").is_ok());

        // Invalid names
        assert!(validate_repository_name("").is_err());
        assert!(validate_repository_name("my repo").is_err()); // spaces not allowed
        assert!(validate_repository_name("repo'; DROP TABLE--").is_err());
        assert!(validate_repository_name("../../../etc/passwd").is_err());
        assert!(validate_repository_name(&"a".repeat(65)).is_err()); // too long
    }

    #[test]
    fn test_validate_file_path() {
        // Valid paths (assuming these directories exist in test environment)
        assert!(validate_file_path("/tmp/test.txt").is_ok());
        
        // Invalid paths
        assert!(validate_file_path("").is_err());
        assert!(validate_file_path("../../../etc/passwd").is_err());
        assert!(validate_file_path("/tmp/../../../etc/passwd").is_err());
        assert!(validate_file_path("~/sensitive_file").is_err());
        assert!(validate_file_path("/tmp/file\0name").is_err());
    }

    #[test]
    fn test_validate_server_address() {
        // Valid addresses
        assert!(validate_server_address("api.example.com").is_ok());
        assert!(validate_server_address("server-123.test.io").is_ok());
        assert!(validate_server_address("192.168.1.1").is_ok());

        // Invalid addresses
        assert!(validate_server_address("").is_err());
        assert!(validate_server_address("server with spaces.com").is_err());
        assert!(validate_server_address("-invalid.com").is_err());
        assert!(validate_server_address("invalid-.com").is_err());
        assert!(validate_server_address("server.com'; DROP TABLE--").is_err());
        assert!(validate_server_address(&"a".repeat(254)).is_err());
    }

    #[test]
    fn test_validate_field_name() {
        // Valid names
        assert!(validate_field_name("FieldName").is_ok());
        assert!(validate_field_name("field_name_123").is_ok());
        assert!(validate_field_name("Field Name With Spaces").is_ok());

        // Invalid names
        assert!(validate_field_name("").is_err());
        assert!(validate_field_name("123field").is_err()); // must start with letter
        assert!(validate_field_name("field'; DROP TABLE--").is_err());
        assert!(validate_field_name("<script>alert('xss')</script>").is_err());
        assert!(validate_field_name(&"a".repeat(129)).is_err());
    }

    #[test]
    fn test_validate_field_value() {
        // Valid values
        assert!(validate_field_value("Normal text value").is_ok());
        assert!(validate_field_value("Value with 'quotes'").is_ok());
        
        // Script injection attempts should be rejected
        assert!(validate_field_value("<script>alert('xss')</script>").is_err());
        assert!(validate_field_value("javascript:void(0)").is_err());
        
        // SQL-like content should be sanitized but allowed
        let result = validate_field_value("O'Brien's value");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "O''Brien''s value");

        // Too long value
        let long_value = "a".repeat(MAX_FIELD_VALUE_LENGTH + 1);
        assert!(validate_field_value(&long_value).is_err());
    }

    #[test]
    fn test_validate_file_name() {
        // Valid names
        assert!(validate_file_name("document.pdf").is_ok());
        assert!(validate_file_name("my-file_123.txt").is_ok());
        assert!(validate_file_name("report.2024.docx").is_ok());

        // Invalid names
        assert!(validate_file_name("").is_err());
        assert!(validate_file_name("../../../etc/passwd").is_err());
        assert!(validate_file_name("file/with/path.txt").is_err());
        assert!(validate_file_name("file\\with\\path.txt").is_err());
        assert!(validate_file_name("file\0name.txt").is_err());
        
        if cfg!(windows) {
            assert!(validate_file_name("CON").is_err());
            assert!(validate_file_name("PRN.txt").is_err());
            assert!(validate_file_name("file:name.txt").is_err());
        }
    }

    #[test]
    fn test_validate_file_size() {
        // Valid sizes
        assert!(validate_file_size(1024).is_ok());
        assert!(validate_file_size(MAX_FILE_SIZE).is_ok());

        // Invalid sizes
        assert!(validate_file_size(MAX_FILE_SIZE + 1).is_err());
    }

    #[test]
    fn test_validate_metadata_json() {
        // Valid metadata
        let valid_json = serde_json::json!({
            "Title": "Test Document",
            "Author": "John Doe",
            "Year": 2024
        });
        assert!(validate_metadata_json(&valid_json).is_ok());

        // Metadata with SQL injection attempt in field name
        let invalid_json = serde_json::json!({
            "Title'; DROP TABLE--": "Test",
            "Author": "John Doe"
        });
        assert!(validate_metadata_json(&invalid_json).is_err());

        // Metadata with script injection in value
        let script_json = serde_json::json!({
            "Title": "<script>alert('xss')</script>",
            "Author": "John Doe"
        });
        assert!(validate_metadata_json(&script_json).is_err());
    }

    #[test]
    fn test_sql_injection_patterns() {
        let test_cases = vec![
            ("SELECT * FROM users", true),
            ("'; DROP TABLE users--", true),
            ("1' OR '1'='1", true),
            ("normal text", false),
            ("UNION SELECT password", true),
            ("INSERT INTO table", true),
        ];

        for (input, should_match) in test_cases {
            assert_eq!(
                SQL_INJECTION_PATTERN.is_match(input),
                should_match,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_script_injection_patterns() {
        let test_cases = vec![
            ("<script>alert('xss')</script>", true),
            ("javascript:void(0)", true),
            ("onclick='alert()'", true),
            ("normal text", false),
            ("document.cookie", true),
            ("window.location", true),
        ];

        for (input, should_match) in test_cases {
            assert_eq!(
                SCRIPT_INJECTION_PATTERN.is_match(input),
                should_match,
                "Failed for input: {}",
                input
            );
        }
    }
}