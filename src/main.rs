// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.

use laserfiche_rs::{laserfiche, config};
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables with proper validation
    let config = match config::Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            eprintln!("\nPlease set the following environment variables:");
            eprintln!("  - LF_API_ADDRESS: The Laserfiche API server address");
            eprintln!("  - LF_REPOSITORY: The repository name");
            eprintln!("  - LF_USERNAME: Your username");
            eprintln!("  - LF_PASSWORD: Your password");
            eprintln!("\nNote: Placeholder values like 'username' or 'your-server' are not allowed.");
            process::exit(1);
        }
    };
    
    // Initialize API configuration from validated environment variables
    let api_server = laserfiche::LFApiServer {
        address: config.api_address.clone(),
        repository: config.repository.clone(),
    };
    
    // Authenticate with the API using validated credentials
    let auth_result = laserfiche::Auth::new(
        api_server.clone(),
        config.username,
        config.password
    ).await?;

    // Handle authentication result
    let _auth = match auth_result {
        laserfiche::AuthOrError::Auth(auth) => {
            println!("Authentication successful!");
            auth
        },
        laserfiche::AuthOrError::LFAPIError(error) => {
            eprintln!("Authentication failed: {:?}", error);
            return Err(format!("Authentication failed: {:?}", error).into());
        }
    };

    // Run example tests (uncomment as needed)
    // test_token_refresh(&auth).await?;
    // test_file_import(&api_server, &auth).await?;
    // test_list_entries(&api_server, &auth).await?;
    // test_export_file(&api_server, &auth).await?;
    
    println!("\nAll tests completed successfully!");
    Ok(())
}

#[allow(dead_code)]
async fn test_token_refresh(auth: &laserfiche::Auth) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting token refresh...");
    
    let refresh_result = auth.refresh().await?;
    
    match refresh_result {
        laserfiche::AuthOrError::Auth(refreshed_auth) => {
            println!("Token refreshed successfully: {:?}", refreshed_auth);
        },
        laserfiche::AuthOrError::LFAPIError(error) => {
            eprintln!("Token refresh failed: {:?}", error);
            return Err(format!("Token refresh failed: {:?}", error).into());
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn test_file_import(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting file import...");
    
    let import_result = laserfiche::Entry::import(
        api_server.clone(),
        auth.clone(),
        "incoming".to_string(),
        "test2.tiff".to_string(),
        1  // Parent folder ID
    ).await?;
    
    match import_result {
        laserfiche::ImportResultOrError::ImportResult(result) => {
            println!("File imported successfully: {:?}", result);
        },
        laserfiche::ImportResultOrError::LFAPIError(error) => {
            eprintln!("File import failed: {:?}", error);
            return Err(format!("File import failed: {:?}", error).into());
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn test_list_entries(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nListing entries...");
    
    let entries_result = laserfiche::Entry::list(
        api_server.clone(),
        auth.clone(),
        1  // Folder ID
    ).await?;
    
    match entries_result {
        laserfiche::EntriesOrError::Entries(entries) => {
            println!("Found {} entries", entries.value.len());
            
            // Get metadata for each entry
            for entry in entries.value {
                if let Err(e) = get_entry_metadata(api_server, auth, entry.id).await {
                    eprintln!("Failed to get metadata for entry {}: {}", entry.id, e);
                }
            }
        },
        laserfiche::EntriesOrError::LFAPIError(error) => {
            eprintln!("Failed to list entries: {:?}", error);
            return Err(format!("Failed to list entries: {:?}", error).into());
        }
    }
    Ok(())
}

async fn get_entry_metadata(
    api_server: &laserfiche::LFApiServer,
    auth: &laserfiche::Auth,
    entry_id: i64
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata_result = laserfiche::Entry::get_metadata(
        api_server.clone(),
        auth.clone(),
        entry_id
    ).await?;
    
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
            return Err(format!("Failed to get metadata for entry {}: {:?}", entry_id, error).into());
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn test_export_file(api_server: &laserfiche::LFApiServer, auth: &laserfiche::Auth) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nExporting file...");
    
    let export_result = laserfiche::Entry::export(
        api_server.clone(),
        auth.clone(),
        3740,  // Entry ID to export
        "export_test.png"
    ).await?;
    
    match export_result {
        laserfiche::BitsOrError::Bits(data) => {
            println!("File exported successfully: {} bytes", data.len());
        },
        laserfiche::BitsOrError::LFAPIError(error) => {
            eprintln!("Export failed: {:?}", error);
            return Err(format!("Export failed: {:?}", error).into());
        }
    }
    Ok(())
}