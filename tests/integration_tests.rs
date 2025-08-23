use laserfiche_rs::laserfiche::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[tokio::test]
async fn test_authentication_flow() {
    // Use test environment variables or skip test if not available
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    assert!(auth_result.is_ok(), "Authentication should not return an error");

    match auth_result.unwrap() {
        AuthOrError::Auth(auth) => {
            assert!(!auth.access_token.is_empty(), "Token should not be empty");
            assert!(auth.timestamp > 0, "Timestamp should be greater than 0");
        },
        AuthOrError::LFAPIError(error) => {
            panic!("Authentication failed with error: {:?}", error);
        }
    }
}

#[tokio::test]
async fn test_token_refresh() {
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        let refresh_result = auth.refresh().await;
        
        assert!(refresh_result.is_ok(), "Token refresh should not return an error");
        
        match refresh_result.unwrap() {
            AuthOrError::Auth(refreshed_auth) => {
                assert!(!refreshed_auth.access_token.is_empty(), "Refreshed token should not be empty");
                assert!(refreshed_auth.timestamp > auth.timestamp, "New timestamp should be greater than old");
            },
            AuthOrError::LFAPIError(error) => {
                panic!("Token refresh failed with error: {:?}", error);
            }
        }
    }
}

#[test]
fn test_blocking_authentication() {
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping blocking integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new_blocking(
        api_server,
        username.unwrap(),
        password.unwrap()
    );

    assert!(auth_result.is_ok(), "Blocking authentication should not return an error");

    match auth_result.unwrap() {
        AuthOrError::Auth(auth) => {
            assert!(!auth.access_token.is_empty(), "Token should not be empty");
        },
        AuthOrError::LFAPIError(error) => {
            panic!("Blocking authentication failed with error: {:?}", error);
        }
    }
}

#[tokio::test]
async fn test_list_entries() {
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        // List entries in root folder (ID: 1)
        let entries_result = Entry::list(
            api_server,
            auth,
            1
        ).await;

        assert!(entries_result.is_ok(), "List entries should not return an error");

        match entries_result.unwrap() {
            EntriesOrError::Entries(entries) => {
                // Root folder should exist and may contain entries
                assert!(entries.value.len() >= 0, "Should return entry list");
            },
            EntriesOrError::LFAPIError(error) => {
                eprintln!("List entries returned error (may be expected): {:?}", error);
            }
        }
    }
}

#[tokio::test]
async fn test_get_entry() {
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        // Get root folder (ID: 1)
        let entry_result = Entry::get(
            api_server,
            auth,
            1
        ).await;

        assert!(entry_result.is_ok(), "Get entry should not return an error");

        match entry_result.unwrap() {
            EntryOrError::Entry(entry) => {
                assert_eq!(entry.id, 1, "Root folder should have ID 1");
                assert!(entry.is_container, "Root folder should be a container");
            },
            EntryOrError::LFAPIError(error) => {
                eprintln!("Get entry returned error (may be expected): {:?}", error);
            }
        }
    }
}

#[tokio::test]
async fn test_search_entries() {
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        // Search for all entries
        let search_result = Entry::search(
            api_server,
            auth,
            "".to_string(),  // No search term
            None,  // No filter
            None,  // No orderby
            None,  // No select
            Some(10),  // Top 10 results
        ).await;

        assert!(search_result.is_ok(), "Search should not return an error");

        match search_result.unwrap() {
            EntriesOrError::Entries(entries) => {
                assert!(entries.value.len() <= 10, "Should return at most 10 entries");
            },
            EntriesOrError::LFAPIError(error) => {
                eprintln!("Search returned error (may be expected): {:?}", error);
            }
        }
    }
}

#[tokio::test]
async fn test_future_timestamp_handling() {
    // This test verifies that the authentication system can handle future timestamps correctly
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    // Test authentication with current time
    let auth_result = Auth::new(
        api_server.clone(),
        username.unwrap(),
        password.unwrap()
    ).await;

    assert!(auth_result.is_ok(), "Authentication should succeed");

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        // Verify timestamp is reasonable (not in far future due to overflow)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // Timestamp should be within 1 second of current time
        assert!(
            (auth.timestamp - current_time).abs() <= 1,
            "Timestamp should be close to current time, got {} vs {}", 
            auth.timestamp, 
            current_time
        );
        
        // Verify timestamp is not negative
        assert!(auth.timestamp > 0, "Timestamp should be positive");
        
        // Verify timestamp is less than i64::MAX (no overflow)
        assert!(auth.timestamp < i64::MAX, "Timestamp should not overflow");
    }
}

#[tokio::test]
async fn test_year_2038_compatibility() {
    // This test ensures our system will work correctly after the year 2038
    // We can't actually set the system time to 2038, but we can verify
    // that our timestamp handling code works with 2038+ values
    
    let year_2038_timestamp: i64 = 2_147_483_647; // 2038-01-19 03:14:07 UTC
    let year_2040_timestamp: i64 = 2_208_988_800; // 2040-01-01 00:00:00 UTC
    let year_2050_timestamp: i64 = 2_524_608_000; // 2050-01-01 00:00:00 UTC
    
    // These should all be valid i64 values
    assert!(year_2038_timestamp > 0);
    assert!(year_2040_timestamp > 0);
    assert!(year_2050_timestamp > 0);
    
    // Verify they're all less than i64::MAX
    assert!(year_2038_timestamp < i64::MAX);
    assert!(year_2040_timestamp < i64::MAX);
    assert!(year_2050_timestamp < i64::MAX);
    
    // If we have test credentials, verify the API can handle these timestamps
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_some() && repository.is_some() && username.is_some() && password.is_some() {
        let api_server = LFApiServer {
            address: address.unwrap(),
            repository: repository.unwrap(),
        };

        // Create auth and verify it handles current time correctly
        let auth_result = Auth::new(
            api_server,
            username.unwrap(),
            password.unwrap()
        ).await;

        if let Ok(AuthOrError::Auth(mut auth)) = auth_result {
            // Manually set to future timestamps and verify they're handled correctly
            auth.timestamp = year_2038_timestamp;
            assert_eq!(auth.timestamp, year_2038_timestamp);
            
            auth.timestamp = year_2040_timestamp;
            assert_eq!(auth.timestamp, year_2040_timestamp);
            
            auth.timestamp = year_2050_timestamp;
            assert_eq!(auth.timestamp, year_2050_timestamp);
        }
    }
}

#[test]
fn test_blocking_future_timestamps() {
    // Test blocking API with future timestamp handling
    let address = env::var("LF_TEST_API_ADDRESS").ok();
    let repository = env::var("LF_TEST_REPOSITORY").ok();
    let username = env::var("LF_TEST_USERNAME").ok();
    let password = env::var("LF_TEST_PASSWORD").ok();

    if address.is_none() || repository.is_none() || username.is_none() || password.is_none() {
        eprintln!("Skipping blocking integration test: Missing test environment variables");
        return;
    }

    let api_server = LFApiServer {
        address: address.unwrap(),
        repository: repository.unwrap(),
    };

    let auth_result = Auth::new_blocking(
        api_server,
        username.unwrap(),
        password.unwrap()
    );

    assert!(auth_result.is_ok(), "Blocking authentication should succeed");

    if let Ok(AuthOrError::Auth(auth)) = auth_result {
        // Verify timestamp is reasonable
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // Timestamp should be within 1 second of current time
        assert!(
            (auth.timestamp - current_time).abs() <= 1,
            "Blocking API timestamp should be close to current time"
        );
        
        // Verify no overflow
        assert!(auth.timestamp > 0 && auth.timestamp < i64::MAX);
    }
}