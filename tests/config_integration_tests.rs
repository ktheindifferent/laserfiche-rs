use laserfiche_rs::config::Config;
use std::env;
use std::process::Command;

#[test]
fn test_application_fails_without_env_vars() {
    // Clear all environment variables
    env::remove_var("LF_API_ADDRESS");
    env::remove_var("LF_REPOSITORY");
    env::remove_var("LF_USERNAME");
    env::remove_var("LF_PASSWORD");
    
    // Try to load config - should fail
    let result = Config::from_env();
    assert!(result.is_err(), "Config should fail without environment variables");
    
    // Verify the error message is helpful
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("Required environment variable"), 
            "Error message should mention missing environment variable");
}

#[test]
fn test_application_rejects_hardcoded_defaults() {
    // Set hardcoded default values that should be rejected
    env::set_var("LF_API_ADDRESS", "your-server.laserfiche.com");
    env::set_var("LF_REPOSITORY", "your-repository");
    env::set_var("LF_USERNAME", "username");
    env::set_var("LF_PASSWORD", "password");
    
    let result = Config::from_env();
    assert!(result.is_err(), "Config should reject hardcoded default values");
    
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("placeholder"), 
            "Error message should mention placeholder values");
    
    // Clean up
    env::remove_var("LF_API_ADDRESS");
    env::remove_var("LF_REPOSITORY");
    env::remove_var("LF_USERNAME");
    env::remove_var("LF_PASSWORD");
}

#[test]
fn test_application_rejects_partial_placeholders() {
    // Test various placeholder patterns
    let test_cases = vec![
        ("your-custom-server.com", "LF_API_ADDRESS"),
        ("example.com", "LF_API_ADDRESS"),
        ("placeholder-repo", "LF_REPOSITORY"),
        ("test", "LF_USERNAME"),
        ("default", "LF_PASSWORD"),
    ];
    
    for (value, var_name) in test_cases {
        // Set up valid env vars except for the one being tested
        env::set_var("LF_API_ADDRESS", "valid.server.com");
        env::set_var("LF_REPOSITORY", "valid-repo");
        env::set_var("LF_USERNAME", "valid-user");
        env::set_var("LF_PASSWORD", "valid-pass");
        
        // Override with the test value
        env::set_var(var_name, value);
        
        let result = Config::from_env();
        assert!(result.is_err(), 
                "Config should reject placeholder value '{}' for {}", value, var_name);
        
        // Clean up
        env::remove_var("LF_API_ADDRESS");
        env::remove_var("LF_REPOSITORY");
        env::remove_var("LF_USERNAME");
        env::remove_var("LF_PASSWORD");
    }
}

#[test]
fn test_no_hardcoded_credentials_in_source() {
    // This test scans the main.rs file to ensure no hardcoded credentials exist
    let main_content = std::fs::read_to_string("src/main.rs")
        .expect("Should be able to read main.rs");
    
    // Check for hardcoded fallback patterns that were in the original code
    assert!(!main_content.contains("unwrap_or_else(|_| \"your-server.laserfiche.com\""),
            "main.rs should not contain hardcoded server fallback");
    assert!(!main_content.contains("unwrap_or_else(|_| \"your-repository\""),
            "main.rs should not contain hardcoded repository fallback");
    assert!(!main_content.contains("unwrap_or_else(|_| \"username\""),
            "main.rs should not contain hardcoded username fallback");
    assert!(!main_content.contains("unwrap_or_else(|_| \"password\""),
            "main.rs should not contain hardcoded password fallback");
    
    // Check for any unwrap_or patterns with string literals (potential hardcoded values)
    let lines_with_unwrap_or: Vec<_> = main_content
        .lines()
        .filter(|line| line.contains("unwrap_or"))
        .collect();
    
    for line in lines_with_unwrap_or {
        // Make sure no unwrap_or is used with credential-like strings
        assert!(!line.contains("laserfiche.com"), 
                "Line should not have hardcoded laserfiche.com: {}", line);
        assert!(!line.contains("password"), 
                "Line should not have hardcoded password: {}", line);
        assert!(!line.contains("username"), 
                "Line should not have hardcoded username: {}", line);
    }
}

#[test]
fn test_main_exits_with_error_on_missing_config() {
    // This test would run the actual binary, but we'll skip it in unit tests
    // as it requires building the binary first. This is more of an integration test
    // that could be run in CI/CD pipeline.
    
    // For now, we just verify that the config module exists and has proper error types
    let config_content = std::fs::read_to_string("src/config.rs")
        .expect("Should be able to read config.rs");
    
    assert!(config_content.contains("ConfigError"), 
            "config.rs should define ConfigError type");
    assert!(config_content.contains("MissingEnvVar"), 
            "config.rs should handle missing environment variables");
    assert!(config_content.contains("InvalidValue"), 
            "config.rs should handle invalid values");
}

#[test]
fn test_config_validates_all_required_fields() {
    // Test that all required fields are checked
    let required_vars = vec![
        "LF_API_ADDRESS",
        "LF_REPOSITORY", 
        "LF_USERNAME",
        "LF_PASSWORD"
    ];
    
    for missing_var in &required_vars {
        // Set all vars except the one we're testing
        for var in &required_vars {
            if var != missing_var {
                env::set_var(var, "valid-value-123");
            } else {
                env::remove_var(var);
            }
        }
        
        let result = Config::from_env();
        assert!(result.is_err(), 
                "Config should fail when {} is missing", missing_var);
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains(missing_var), 
                "Error message should mention the missing variable: {}", missing_var);
        
        // Clean up
        for var in &required_vars {
            env::remove_var(var);
        }
    }
}