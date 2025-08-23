use std::env;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    InvalidValue(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => {
                write!(f, "Required environment variable '{}' is not set", var)
            }
            ConfigError::InvalidValue(msg) => {
                write!(f, "Invalid configuration value: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone)]
pub struct Config {
    pub api_address: String,
    pub repository: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_address = env::var("LF_API_ADDRESS")
            .map_err(|_| ConfigError::MissingEnvVar("LF_API_ADDRESS".to_string()))?;
        
        let repository = env::var("LF_REPOSITORY")
            .map_err(|_| ConfigError::MissingEnvVar("LF_REPOSITORY".to_string()))?;
        
        let username = env::var("LF_USERNAME")
            .map_err(|_| ConfigError::MissingEnvVar("LF_USERNAME".to_string()))?;
        
        let password = env::var("LF_PASSWORD")
            .map_err(|_| ConfigError::MissingEnvVar("LF_PASSWORD".to_string()))?;
        
        Self::validate_not_placeholder(&api_address, "LF_API_ADDRESS")?;
        Self::validate_not_placeholder(&repository, "LF_REPOSITORY")?;
        Self::validate_not_placeholder(&username, "LF_USERNAME")?;
        Self::validate_not_placeholder(&password, "LF_PASSWORD")?;
        
        Ok(Config {
            api_address,
            repository,
            username,
            password,
        })
    }
    
    fn validate_not_placeholder(value: &str, var_name: &str) -> Result<(), ConfigError> {
        let invalid_values = [
            "your-server.laserfiche.com",
            "your-repository",
            "username",
            "password",
            "placeholder",
            "default",
            "example",
            "test",
            "",
        ];
        
        let normalized_value = value.trim().to_lowercase();
        
        if invalid_values.iter().any(|&invalid| normalized_value == invalid) {
            return Err(ConfigError::InvalidValue(
                format!("{} contains a placeholder or default value: '{}'", var_name, value)
            ));
        }
        
        if normalized_value.contains("your-") || 
           normalized_value.contains("example") || 
           normalized_value.contains("placeholder") {
            return Err(ConfigError::InvalidValue(
                format!("{} appears to contain a placeholder value: '{}'", var_name, value)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    fn clear_env_vars() {
        env::remove_var("LF_API_ADDRESS");
        env::remove_var("LF_REPOSITORY");
        env::remove_var("LF_USERNAME");
        env::remove_var("LF_PASSWORD");
    }
    
    #[test]
    fn test_missing_env_vars() {
        clear_env_vars();
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::MissingEnvVar(var) => {
                assert_eq!(var, "LF_API_ADDRESS");
            }
            _ => panic!("Expected MissingEnvVar error"),
        }
    }
    
    #[test]
    fn test_all_env_vars_missing() {
        clear_env_vars();
        
        let result = Config::from_env();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_placeholder_values_rejected() {
        clear_env_vars();
        
        env::set_var("LF_API_ADDRESS", "your-server.laserfiche.com");
        env::set_var("LF_REPOSITORY", "valid-repo");
        env::set_var("LF_USERNAME", "valid-user");
        env::set_var("LF_PASSWORD", "valid-pass");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::InvalidValue(msg) => {
                assert!(msg.contains("placeholder or default value"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
        
        clear_env_vars();
    }
    
    #[test]
    fn test_default_username_rejected() {
        clear_env_vars();
        
        env::set_var("LF_API_ADDRESS", "api.laserfiche.com");
        env::set_var("LF_REPOSITORY", "myrepo");
        env::set_var("LF_USERNAME", "username");
        env::set_var("LF_PASSWORD", "mypassword");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::InvalidValue(msg) => {
                assert!(msg.contains("LF_USERNAME"));
                assert!(msg.contains("placeholder or default value"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
        
        clear_env_vars();
    }
    
    #[test]
    fn test_valid_config() {
        clear_env_vars();
        
        env::set_var("LF_API_ADDRESS", "api.laserfiche.com");
        env::set_var("LF_REPOSITORY", "production-repo");
        env::set_var("LF_USERNAME", "john.doe");
        env::set_var("LF_PASSWORD", "secure123!");
        
        let result = Config::from_env();
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.api_address, "api.laserfiche.com");
        assert_eq!(config.repository, "production-repo");
        assert_eq!(config.username, "john.doe");
        assert_eq!(config.password, "secure123!");
        
        clear_env_vars();
    }
    
    #[test]
    fn test_empty_values_rejected() {
        clear_env_vars();
        
        env::set_var("LF_API_ADDRESS", "");
        env::set_var("LF_REPOSITORY", "repo");
        env::set_var("LF_USERNAME", "user");
        env::set_var("LF_PASSWORD", "pass");
        
        let result = Config::from_env();
        assert!(result.is_err());
        
        clear_env_vars();
    }
}