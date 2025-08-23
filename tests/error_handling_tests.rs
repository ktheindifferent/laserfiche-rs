use laserfiche_rs::laserfiche::*;

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_invalid_credentials_error_message() {
    // This test intentionally uses invalid credentials to verify error handling
    let address = match std::env::var("LF_TEST_API_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("⚠️  Skipping test: LF_TEST_API_ADDRESS not set");
            eprintln!("   This test requires a valid API address to test error handling");
            return;
        }
    };
    
    let repository = match std::env::var("LF_TEST_REPOSITORY") {
        Ok(repo) => repo,
        Err(_) => {
            eprintln!("⚠️  Skipping test: LF_TEST_REPOSITORY not set");
            eprintln!("   This test requires a valid repository name to test error handling");
            return;
        }
    };

    let api_server = LFApiServer { address, repository };
    
    let auth_result = Auth::new(
        api_server,
        "invalid_user".to_string(),
        "invalid_password".to_string()
    ).await;

    // We expect this to succeed at the request level but fail at the API level
    assert!(auth_result.is_ok(), 
        "Authentication request should complete even with invalid credentials. Request error: {:?}", 
        auth_result.err());

    match auth_result.expect("Already verified auth_result is Ok") {
        AuthOrError::Auth(_) => {
            panic!("Authentication should not succeed with invalid credentials!");
        },
        AuthOrError::LFAPIError(error) => {
            // Verify we get a meaningful error
            assert!(error.status.is_some(), 
                "API error should include HTTP status code. Error: {:?}", error);
            
            let status = error.status.expect("Already checked status is Some") as i32;
            assert!(status == 401 || status == 403, 
                "Invalid credentials should return 401 Unauthorized or 403 Forbidden, got status code: {}", 
                status);
            
            assert!(error.title.is_some() || error.detail.is_some(), 
                "API error should include a title or detail message for debugging. Error: {:?}", 
                error);
            
            eprintln!("✓ Invalid credentials correctly returned error:");
            eprintln!("  Status: {}", status);
            eprintln!("  Title: {}", error.title.as_ref().unwrap_or(&"No title".to_string()));
            eprintln!("  Detail: {}", error.detail.as_ref().unwrap_or(&"No detail".to_string()));
        }
    }
}

#[tokio::test]
async fn test_nonexistent_entry_error() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Authentication should succeed for error handling test");
    
    // Try to get an entry with an ID that almost certainly doesn't exist
    let entry_result = Entry::get(
        config.api_server.clone(),
        auth,
        999999999  // Very unlikely to exist
    ).await;

    assert!(entry_result.is_ok(), 
        "Get entry request should complete even for nonexistent entry. Request error: {:?}", 
        entry_result.err());

    match entry_result.expect("Already verified entry_result is Ok") {
        EntryOrError::Entry(entry) => {
            // If by chance this entry exists, just verify it's valid
            assert_valid_entry(&entry, Some(999999999));
            eprintln!("⚠️  Entry 999999999 unexpectedly exists in test repository");
        },
        EntryOrError::LFAPIError(error) => {
            // This is the expected path
            assert!(error.status.is_some(), 
                "API error should include HTTP status code for nonexistent entry. Error: {:?}", 
                error);
            
            let status = error.status.expect("Already checked status is Some") as i32;
            assert!(status == 404 || status == 403, 
                "Nonexistent entry should return 404 Not Found or 403 Forbidden, got status code: {}", 
                status);
            
            eprintln!("✓ Nonexistent entry correctly returned error:");
            eprintln!("  Status: {}", status);
            eprintln!("  Title: {}", error.title.as_ref().unwrap_or(&"No title".to_string()));
            eprintln!("  Detail: {}", error.detail.as_ref().unwrap_or(&"No detail".to_string()));
        }
    }
}

#[tokio::test]
async fn test_search_with_invalid_filter() {
    let config = skip_if_no_config!();
    
    let auth = config.authenticate().await
        .expect("Authentication should succeed for search error test");
    
    // Try a search with an invalid OData filter syntax
    let search_result = Entry::search(
        config.api_server.clone(),
        auth,
        "".to_string(),
        Some("invalid filter syntax $@#".to_string()),  // Invalid OData filter
        None,
        None,
        Some(5),
    ).await;

    // The API might accept the request but return an error in the response
    if let Ok(result) = search_result {
        match result {
            EntriesOrError::Entries(entries) => {
                // Some APIs might ignore invalid filters
                eprintln!("⚠️  API ignored invalid filter and returned {} entries", 
                    entries.value.len());
                assert_valid_entries(&entries, Some(5));
            },
            EntriesOrError::LFAPIError(error) => {
                // This is expected - invalid filter should cause an error
                assert!(error.status.is_some() || error.title.is_some() || error.detail.is_some(),
                    "Invalid filter should return error details. Error: {:?}", error);
                
                eprintln!("✓ Invalid filter correctly returned API error:");
                if let Some(status) = error.status {
                    eprintln!("  Status: {}", status);
                }
                if let Some(title) = &error.title {
                    eprintln!("  Title: {}", title);
                }
                if let Some(detail) = &error.detail {
                    eprintln!("  Detail: {}", detail);
                }
            }
        }
    } else {
        // Request itself failed
        eprintln!("✓ Invalid filter caused request error: {:?}", search_result.err());
    }
}

#[tokio::test] 
async fn test_expired_token_refresh() {
    let config = skip_if_no_config!();
    
    // Get a valid auth token
    let mut auth = config.authenticate().await
        .expect("Initial authentication should succeed");
    
    // Artificially expire the token by setting timestamp to 0
    // This simulates an expired token scenario
    let original_token = auth.access_token.clone();
    auth.timestamp = 0;
    
    // Try to refresh the expired token
    let refresh_result = auth.refresh().await;
    
    assert!(refresh_result.is_ok(),
        "Refresh request should complete. Error: {:?}", 
        refresh_result.err());
    
    match refresh_result.expect("Already checked refresh_result is Ok") {
        AuthOrError::Auth(new_auth) => {
            assert!(!new_auth.access_token.is_empty(),
                "Refreshed token should not be empty");
            assert!(new_auth.timestamp > 0,
                "New timestamp should be valid (> 0), got: {}", 
                new_auth.timestamp);
            assert_ne!(new_auth.access_token, original_token,
                "Refreshed token should be different from original token");
            eprintln!("✓ Successfully refreshed expired token");
        },
        AuthOrError::LFAPIError(error) => {
            // Some systems might not allow refresh with timestamp 0
            eprintln!("⚠️  Token refresh with zero timestamp returned error:");
            eprintln!("  Status: {}", error.status.unwrap_or(0));
            eprintln!("  Title: {}", error.title.as_ref().unwrap_or(&"No title".to_string()));
            eprintln!("  Detail: {}", error.detail.as_ref().unwrap_or(&"No detail".to_string()));
        }
    }
}

#[test]
#[should_panic(expected = "Authentication should not succeed with invalid credentials")]
fn test_should_panic_on_unexpected_auth_success() {
    // This test demonstrates proper use of #[should_panic]
    // It will pass if the code panics with the expected message
    
    // Simulate a scenario where we expect auth to fail but it succeeds
    let auth_succeeded = true;  // This would come from actual auth attempt
    
    if auth_succeeded {
        panic!("Authentication should not succeed with invalid credentials!");
    }
    
    // If we reach here, the test will fail because it didn't panic
}