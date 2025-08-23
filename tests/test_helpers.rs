use laserfiche_rs::laserfiche::*;
use std::env;
use std::fmt;

/// Test configuration structure containing all required environment variables
#[derive(Clone, Debug)]
pub struct TestConfig {
    pub api_server: LFApiServer,
    pub username: String,
    pub password: String,
}

impl TestConfig {
    /// Load test configuration from environment variables
    /// Returns None with a descriptive message if any required variables are missing
    pub fn from_env() -> std::result::Result<Self, TestConfigError> {
        let address = env::var("LF_TEST_API_ADDRESS")
            .map_err(|_| TestConfigError::MissingEnvVar("LF_TEST_API_ADDRESS"))?;
        
        let repository = env::var("LF_TEST_REPOSITORY")
            .map_err(|_| TestConfigError::MissingEnvVar("LF_TEST_REPOSITORY"))?;
        
        let username = env::var("LF_TEST_USERNAME")
            .map_err(|_| TestConfigError::MissingEnvVar("LF_TEST_USERNAME"))?;
        
        let password = env::var("LF_TEST_PASSWORD")
            .map_err(|_| TestConfigError::MissingEnvVar("LF_TEST_PASSWORD"))?;

        Ok(TestConfig {
            api_server: LFApiServer { address, repository },
            username,
            password,
        })
    }

    /// Create an authenticated connection for testing
    pub async fn authenticate(&self) -> std::result::Result<Auth, String> {
        let auth_result = match Auth::new(
            self.api_server.clone(),
            self.username.clone(),
            self.password.clone()
        ).await {
            Ok(result) => result,
            Err(e) => return Err(format!("Authentication request failed: {:?}", e)),
        };

        match auth_result {
            AuthOrError::Auth(auth) => Ok(auth),
            AuthOrError::LFAPIError(error) => {
                Err(format!("Authentication failed with API error: {:?}", error))
            }
        }
    }

    /// Create an authenticated connection for blocking tests
    pub fn authenticate_blocking(&self) -> std::result::Result<Auth, String> {
        let auth_result = match Auth::new_blocking(
            self.api_server.clone(),
            self.username.clone(),
            self.password.clone()
        ) {
            Ok(result) => result,
            Err(e) => return Err(format!("Blocking authentication request failed: {:?}", e)),
        };

        match auth_result {
            AuthOrError::Auth(auth) => Ok(auth),
            AuthOrError::LFAPIError(error) => {
                Err(format!("Blocking authentication failed with API error: {:?}", error))
            }
        }
    }
}

#[derive(Debug)]
pub enum TestConfigError {
    MissingEnvVar(&'static str),
}

impl fmt::Display for TestConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestConfigError::MissingEnvVar(var) => {
                write!(f, "Missing required environment variable: {}. Please set this variable to run integration tests.", var)
            }
        }
    }
}

impl std::error::Error for TestConfigError {}

/// Helper macro to skip test with detailed message when config is missing
#[macro_export]
macro_rules! skip_if_no_config {
    () => {
        match TestConfig::from_env() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("⚠️  Skipping test: {}", e);
                eprintln!("   To run this test, set the following environment variables:");
                eprintln!("   - LF_TEST_API_ADDRESS: The Laserfiche API server address");
                eprintln!("   - LF_TEST_REPOSITORY: The repository name");
                eprintln!("   - LF_TEST_USERNAME: Your test username");
                eprintln!("   - LF_TEST_PASSWORD: Your test password");
                return;
            }
        }
    };
}

/// Helper macro to assert authentication success with clear error message
#[macro_export]
macro_rules! assert_auth_success {
    ($auth:expr) => {
        assert!(!$auth.access_token.is_empty(), 
            "Authentication token should not be empty. Received empty token from server.");
        assert!($auth.timestamp > 0, 
            "Authentication timestamp should be greater than 0. Received timestamp: {}", 
            $auth.timestamp);
    };
}

/// Helper function to assert valid entry properties
pub fn assert_valid_entry(entry: &Entry, expected_id: Option<i64>) {
    if let Some(id) = expected_id {
        assert_eq!(entry.id, id, 
            "Entry ID mismatch. Expected: {}, Actual: {}", id, entry.id);
    }
    
    assert!(!entry.name.is_empty() || entry.id == 1, 
        "Entry name should not be empty (except for root folder). Entry ID: {}", entry.id);
}

/// Helper function to assert valid entries collection
pub fn assert_valid_entries(entries: &Entries, max_count: Option<usize>) {
    if let Some(max) = max_count {
        assert!(entries.value.len() <= max,
            "Too many entries returned. Expected at most {}, got {}", 
            max, entries.value.len());
    }
    
    // Verify each entry has basic required fields
    for entry in &entries.value {
        assert_valid_entry(entry, None);
    }
}