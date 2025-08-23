use laserfiche_rs::laserfiche::*;
use std::env;

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