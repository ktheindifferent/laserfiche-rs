// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.

use laserfiche_rs::laserfiche;

#[tokio::main]
async fn main() {
    // Initialize API configuration
    let api_server = laserfiche::LFApiServer {
        address: "<HOST_ADDRESS>".to_string(),
        repository: "<repo>".to_string(),
    };
    
    // Authenticate with the API
    let auth_result = laserfiche::Auth::new(
        api_server.clone(),
        "<repo>".to_string(),
        "<password>".to_string()
    ).await.unwrap();

    // Handle authentication result
    let auth = match auth_result {
        laserfiche::AuthOrError::Auth(auth) => {
            println!("Authentication successful: {:?}", auth);
            auth
        },
        laserfiche::AuthOrError::LFAPIError(error) => {
            eprintln!("Authentication failed: {:?}", error);
            return;
        }
    };

    // Test token refresh
    test_token_refresh(&auth).await;

    // Test file import
    test_file_import(&api_server, &auth).await;
    
    // Uncommented examples for reference:
    // test_list_entries(&api_server, &auth).await;
    // test_export_file(&api_server, &auth).await;
}

async fn test_token_refresh(auth: &laserfiche::Auth) {
    println!("\nTesting token refresh...");
    
    let refresh_result = auth.refresh().await.unwrap();
    
    match refresh_result {
        laserfiche::AuthOrError::Auth(refreshed_auth) => {
            println!("Token refreshed successfully: {:?}", refreshed_auth);
        },
        laserfiche::AuthOrError::LFAPIError(error) => {
            eprintln!("Token refresh failed: {:?}", error);
        }
    }
}

async fn test_file_import(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) {
    println!("\nTesting file import...");
    
    let import_result = laserfiche::Entry::import(
        api_server.clone(),
        auth.clone(),
        "incoming".to_string(),
        "test2.tiff".to_string(),
        1  // Parent folder ID
    ).await.unwrap();
    
    match import_result {
        laserfiche::ImportResultOrError::ImportResult(result) => {
            println!("File imported successfully: {:?}", result);
        },
        laserfiche::ImportResultOrError::LFAPIError(error) => {
            eprintln!("File import failed: {:?}", error);
        }
    }
}

#[allow(dead_code)]
async fn test_list_entries(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) {
    println!("\nListing entries...");
    
    let entries_result = laserfiche::Entry::list(
        api_server.clone(),
        auth.clone(),
        1  // Folder ID
    ).await.unwrap();
    
    match entries_result {
        laserfiche::EntriesOrError::Entries(entries) => {
            println!("Found {} entries", entries.value.len());
            
            // Get metadata for each entry
            for entry in entries.value {
                get_entry_metadata(api_server, auth, entry.id).await;
            }
        },
        laserfiche::EntriesOrError::LFAPIError(error) => {
            eprintln!("Failed to list entries: {:?}", error);
        }
    }
}

async fn get_entry_metadata(
    api_server: &laserfiche::LFApiServer,
    auth: &laserfiche::Auth,
    entry_id: i64
) {
    let metadata_result = laserfiche::Entry::get_metadata(
        api_server.clone(),
        auth.clone(),
        entry_id
    ).await.unwrap();
    
    match metadata_result {
        laserfiche::MetadataResultOrError::Metadata(metadata) => {
            if !metadata.value.is_empty() {
                let first_field = &metadata.value[0];
                println!("Entry {} metadata:", entry_id);
                println!("  Field name: {:?}", first_field.field_name);
                if !first_field.values.is_empty() {
                    println!("  First value: {:?}", first_field.values[0].value);
                }
                println!("  Total fields: {}", metadata.value.len());
            }
        },
        laserfiche::MetadataResultOrError::LFAPIError(error) => {
            eprintln!("Failed to get metadata for entry {}: {:?}", entry_id, error);
        }
    }
}

#[allow(dead_code)]
async fn test_export_file(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) {
    println!("\nExporting file...");
    
    let export_result = laserfiche::Entry::export(
        api_server.clone(),
        auth.clone(),
        3740,  // Entry ID to export
        "export_test.png"
    ).await.unwrap();
    
    match export_result {
        laserfiche::BitsOrError::Bits(data) => {
            println!("File exported successfully: {} bytes", data.len());
        },
        laserfiche::BitsOrError::LFAPIError(error) => {
            eprintln!("Export failed: {:?}", error);
        }
    }
}