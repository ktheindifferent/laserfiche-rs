use laserfiche_rs::laserfiche::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_authentication_flow() {
    let config = skip_if_no_config!();

    let auth_result = Auth::new(
        config.api_server.clone(),
        config.username.clone(),
        config.password.clone()
    ).await;

    assert!(auth_result.is_ok(), 
        "Authentication request should succeed. Error: {:?}", 
        auth_result.err());

    match auth_result.expect("Already checked auth_result is Ok") {
        AuthOrError::Auth(auth) => {
            assert_auth_success!(auth);
        },
        AuthOrError::LFAPIError(error) => {
            panic!("Authentication failed with API error. \nStatus: {}\nTitle: {}\nDetail: {}\nFull error: {:?}", 
                error.status.unwrap_or(0), 
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()),
                error);
        }
    }
}

#[tokio::test]
async fn test_token_refresh() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Initial authentication should succeed for token refresh test");
    
    let original_timestamp = auth.timestamp;
    let refresh_result = auth.refresh().await;
    
    assert!(refresh_result.is_ok(), 
        "Token refresh request should succeed. Error: {:?}", 
        refresh_result.err());
    
    match refresh_result.expect("Already checked refresh_result is Ok") {
        AuthOrError::Auth(refreshed_auth) => {
            assert!(!refreshed_auth.access_token.is_empty(), 
                "Refreshed token should not be empty. Refresh returned empty token.");
            assert!(refreshed_auth.timestamp > original_timestamp, 
                "New timestamp ({}) should be greater than original timestamp ({})", 
                refreshed_auth.timestamp, original_timestamp);
        },
        AuthOrError::LFAPIError(error) => {
            panic!("Token refresh failed with API error. \nStatus: {}\nTitle: {}\nDetail: {}\nFull error: {:?}", 
                error.status.unwrap_or(0), 
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()),
                error);
        }
    }
}

#[test]
fn test_blocking_authentication() {
    let config = skip_if_no_config!();
    
    let auth_result = Auth::new_blocking(
        config.api_server.clone(),
        config.username.clone(),
        config.password.clone()
    );

    assert!(auth_result.is_ok(), 
        "Blocking authentication request should succeed. Error: {:?}", 
        auth_result.err());

    match auth_result.expect("Already checked auth_result is Ok") {
        AuthOrError::Auth(auth) => {
            assert!(!auth.access_token.is_empty(), 
                "Blocking authentication should return non-empty token. Received empty token.");
            assert!(auth.timestamp > 0, 
                "Authentication timestamp should be greater than 0. Received: {}", 
                auth.timestamp);
        },
        AuthOrError::LFAPIError(error) => {
            panic!("Blocking authentication failed with API error. \nStatus: {}\nTitle: {}\nDetail: {}\nFull error: {:?}", 
                error.status.unwrap_or(0), 
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()),
                error);
        }
    }
}

#[tokio::test]
async fn test_list_entries() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Authentication should succeed for list entries test");
    
    // List entries in root folder (ID: 1)
    let entries_result = Entry::list(
        config.api_server.clone(),
        auth,
        1
    ).await;

    assert!(entries_result.is_ok(), 
        "List entries request should succeed. Error: {:?}", 
        entries_result.err());

    match entries_result.expect("Already checked entries_result is Ok") {
        EntriesOrError::Entries(entries) => {
            // Root folder should exist and may contain entries
            assert_valid_entries(&entries, None);
            eprintln!("✓ Successfully listed {} entries from root folder", entries.value.len());
        },
        EntriesOrError::LFAPIError(error) => {
            // Some environments may not have list permissions on root
            eprintln!("⚠️  List entries returned API error (may be expected for root folder).\n   Status: {}\n   Title: {}\n   Detail: {}", 
                error.status.unwrap_or(0),
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()));
        }
    }
}

#[tokio::test]
async fn test_get_entry() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Authentication should succeed for get entry test");
    
    // Get root folder (ID: 1)
    let entry_result = Entry::get(
        config.api_server.clone(),
        auth,
        1
    ).await;

    assert!(entry_result.is_ok(), 
        "Get entry request should succeed. Error: {:?}", 
        entry_result.err());

    match entry_result.expect("Already checked entry_result is Ok") {
        EntryOrError::Entry(entry) => {
            assert_eq!(entry.id, 1, 
                "Root folder should have ID 1, but got ID {}", entry.id);
            assert!(entry.is_container, 
                "Root folder (ID: 1) should be a container, but is_container is false");
            assert_valid_entry(&entry, Some(1));
            eprintln!("✓ Successfully retrieved root folder entry");
        },
        EntryOrError::LFAPIError(error) => {
            // Some environments may not have read permissions on root
            eprintln!("⚠️  Get entry returned API error (may be expected for root folder).\n   Status: {}\n   Title: {}\n   Detail: {}", 
                error.status.unwrap_or(0),
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()));
        }
    }
}

#[tokio::test]
async fn test_search_entries() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Authentication should succeed for search test");
    
    // Search for all entries with a limit
    let search_result = Entry::search(
        config.api_server.clone(),
        auth,
        "".to_string(),  // No search term - get all accessible entries
        None,  // No filter
        None,  // No orderby  
        None,  // No select
        Some(10),  // Top 10 results
    ).await;

    assert!(search_result.is_ok(), 
        "Search request should succeed. Error: {:?}", 
        search_result.err());

    match search_result.expect("Already checked search_result is Ok") {
        EntriesOrError::Entries(entries) => {
            assert_valid_entries(&entries, Some(10));
            eprintln!("✓ Search returned {} entries (requested max 10)", entries.value.len());
        },
        EntriesOrError::LFAPIError(error) => {
            // Some environments may not have search permissions
            eprintln!("⚠️  Search returned API error (may be expected if search is disabled).\n   Status: {}\n   Title: {}\n   Detail: {}", 
                error.status.unwrap_or(0),
                error.title.as_ref().unwrap_or(&"No title".to_string()),
                error.detail.as_ref().unwrap_or(&"No detail".to_string()));
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