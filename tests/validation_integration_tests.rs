// Integration tests for input validation
use laserfiche_rs::laserfiche::{LFApiServer, Auth, Entry, AuthOrError};
use laserfiche_rs::validation;

fn create_test_api_server() -> LFApiServer {
    LFApiServer {
        address: "test.laserfiche.com".to_string(),
        repository: "test-repo".to_string(),
    }
}

#[tokio::test]
async fn test_invalid_entry_id_validation() {
    let api_server = create_test_api_server();
    
    // Create a mock auth (would normally come from Auth::new)
    let auth = match Auth::new(api_server.clone(), "user".to_string(), "pass".to_string()).await {
        Ok(AuthOrError::Auth(auth)) => auth,
        _ => {
            // For testing validation, we'll create a dummy auth
            Auth {
                access_token: "dummy_token".to_string(),
                expires_in: 3600,
                token_type: "Bearer".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                timestamp: 0,
                api_server: api_server.clone(),
                odata_context: String::new(),
            }
        }
    };

    // Test negative entry ID
    let result = Entry::get_metadata(api_server.clone(), auth.clone(), -1).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid entry ID"));

    // Test zero entry ID
    let result = Entry::get_metadata(api_server.clone(), auth.clone(), 0).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid entry ID"));

    // Test extremely large entry ID
    let result = Entry::get_metadata(api_server.clone(), auth.clone(), i64::MAX).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid entry ID"));
}

#[tokio::test]
async fn test_invalid_file_path_validation() {
    let api_server = create_test_api_server();
    let auth = Auth {
        access_token: "dummy_token".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        timestamp: 0,
        api_server: api_server.clone(),
        odata_context: String::new(),
    };

    // Test path traversal attempts
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "../../../etc/passwd".to_string(),
        "test.txt".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Path traversal"));

    // Test null byte in path
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "/tmp/test\0file.txt".to_string(),
        "test.txt".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file path"));

    // Test tilde expansion attempt
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "~/sensitive_file".to_string(),
        "test.txt".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Path traversal"));
}

#[tokio::test]
async fn test_invalid_file_name_validation() {
    let api_server = create_test_api_server();
    let auth = Auth {
        access_token: "dummy_token".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        timestamp: 0,
        api_server: api_server.clone(),
        odata_context: String::new(),
    };

    // Test file name with path traversal
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "/tmp/test.txt".to_string(),
        "../../../etc/passwd".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));

    // Test file name with null byte
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "/tmp/test.txt".to_string(),
        "test\0file.txt".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));

    // Test file name with slashes
    let result = Entry::import(
        api_server.clone(),
        auth.clone(),
        "/tmp/test.txt".to_string(),
        "test/file.txt".to_string(),
        1
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));
}

#[tokio::test]
async fn test_invalid_repository_name_validation() {
    // Test SQL injection in repository name
    let api_server = LFApiServer {
        address: "test.laserfiche.com".to_string(),
        repository: "repo'; DROP TABLE users--".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("SQL injection"));

    // Test repository name with spaces
    let api_server = LFApiServer {
        address: "test.laserfiche.com".to_string(),
        repository: "my repo name".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid repository name"));

    // Test empty repository name
    let api_server = LFApiServer {
        address: "test.laserfiche.com".to_string(),
        repository: "".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid repository name"));
}

#[tokio::test]
async fn test_invalid_server_address_validation() {
    // Test SQL injection in server address
    let api_server = LFApiServer {
        address: "server.com'; DROP TABLE--".to_string(),
        repository: "test-repo".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("SQL injection"));

    // Test server address with spaces
    let api_server = LFApiServer {
        address: "server with spaces.com".to_string(),
        repository: "test-repo".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid URL"));

    // Test empty server address
    let api_server = LFApiServer {
        address: "".to_string(),
        repository: "test-repo".to_string(),
    };
    
    let result = Auth::new(api_server, "user".to_string(), "pass".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid URL"));
}

#[tokio::test]
async fn test_metadata_field_validation() {
    let api_server = create_test_api_server();
    let auth = Auth {
        access_token: "dummy_token".to_string(),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        timestamp: 0,
        api_server: api_server.clone(),
        odata_context: String::new(),
    };

    // Test metadata with SQL injection in field name
    let malicious_metadata = serde_json::json!({
        "Field'; DROP TABLE--": "value"
    });
    
    let result = Entry::update_metadata(
        api_server.clone(),
        auth.clone(),
        1,
        malicious_metadata
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("SQL injection"));

    // Test metadata with script injection in value
    let script_metadata = serde_json::json!({
        "Title": "<script>alert('xss')</script>"
    });
    
    let result = Entry::update_metadata(
        api_server.clone(),
        auth.clone(),
        1,
        script_metadata
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Script injection"));

    // Test metadata with invalid field name starting with number
    let invalid_field_metadata = serde_json::json!({
        "123Field": "value"
    });
    
    let result = Entry::update_metadata(
        api_server.clone(),
        auth.clone(),
        1,
        invalid_field_metadata
    ).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid field name"));
}

#[test]
fn test_file_size_validation() {
    // Test file size exceeding maximum
    let result = validation::validate_file_size(validation::MAX_FILE_SIZE + 1);
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("File size"));

    // Test valid file size
    let result = validation::validate_file_size(1024);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1024);

    // Test maximum allowed file size
    let result = validation::validate_file_size(validation::MAX_FILE_SIZE);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), validation::MAX_FILE_SIZE);
}

#[test]
fn test_url_validation() {
    // Test HTTP URL (should fail, HTTPS required)
    let result = validation::validate_api_url("http://api.example.com");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Insecure URL"));

    // Test invalid URL format
    let result = validation::validate_api_url("not a url");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid URL"));

    // Test URL with SQL injection
    let result = validation::validate_api_url("https://api.com'; DROP TABLE--");
    assert!(result.is_err());
    // URL validation shows "Invalid URL" for malformed URLs, even with SQL injection patterns
    assert!(result.err().unwrap().to_string().contains("Invalid URL"));

    // Test valid HTTPS URL
    let result = validation::validate_api_url("https://api.example.com");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://api.example.com");
}

#[test]
fn test_field_value_sanitization() {
    // Test normal value
    let result = validation::validate_field_value("Normal text value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Normal text value");

    // Test value with single quotes (should be escaped)
    let result = validation::validate_field_value("O'Brien's value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "O''Brien''s value");

    // Test script injection (should fail)
    let result = validation::validate_field_value("<script>alert('xss')</script>");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Script injection"));

    // Test value that's too long
    let long_value = "a".repeat(validation::MAX_FIELD_VALUE_LENGTH + 1);
    let result = validation::validate_field_value(&long_value);
    assert!(result.is_err());
    // The error message shows "Invalid field value: contains potentially malicious content"
    assert!(result.err().unwrap().to_string().contains("Invalid field value"));
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_specific_file_validation() {
    // Test reserved Windows file names
    let result = validation::validate_file_name("CON");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));

    let result = validation::validate_file_name("PRN.txt");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));

    // Test Windows invalid characters
    let result = validation::validate_file_name("file:name.txt");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));

    let result = validation::validate_file_name("file<name>.txt");
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("Invalid file name"));
}