// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.

use laserfiche_rs::laserfiche;
use std::env;
use log::debug;

/// Helper trait for safe array access with logging
trait SafeArrayAccess<T> {
    fn safe_get(&self, index: usize, context: &str) -> Option<&T>;
    fn safe_first(&self, context: &str) -> Option<&T>;
}

impl<T> SafeArrayAccess<T> for Vec<T> {
    fn safe_get(&self, index: usize, context: &str) -> Option<&T> {
        match self.get(index) {
            Some(value) => Some(value),
            None => {
                debug!("Array access at index {} failed for {}: array length is {}", 
                       index, context, self.len());
                None
            }
        }
    }
    
    fn safe_first(&self, context: &str) -> Option<&T> {
        match self.first() {
            Some(value) => Some(value),
            None => {
                debug!("Attempted to access first element of empty array for {}", context);
                None
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize API configuration from environment variables
    let api_server = laserfiche::LFApiServer {
        address: env::var("LF_API_ADDRESS")
            .unwrap_or_else(|_| "your-server.laserfiche.com".to_string()),
        repository: env::var("LF_REPOSITORY")
            .unwrap_or_else(|_| "your-repository".to_string()),
    };
    
    // Authenticate with the API using environment variables
    let username = env::var("LF_USERNAME")
        .unwrap_or_else(|_| "username".to_string());
    let password = env::var("LF_PASSWORD")
        .unwrap_or_else(|_| "password".to_string());
        
    let auth_result = laserfiche::Auth::new(
        api_server.clone(),
        username,
        password
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
            // Use the safe access helper method
            match metadata.value.safe_first(&format!("metadata for entry {}", entry_id)) {
                Some(first_field) => {
                    println!("Entry {} metadata:", entry_id);
                    println!("  Field name: {:?}", first_field.field_name);
                    
                    // Safe array access for values using helper method
                    match first_field.values.safe_first(&format!("values for field '{}'", first_field.field_name)) {
                        Some(first_value) => {
                            println!("  First value: {:?}", first_value.value);
                        },
                        None => {
                            println!("  Field has no values");
                        }
                    }
                    println!("  Total fields: {}", metadata.value.len());
                },
                None => {
                    debug!("Entry {} has no metadata fields", entry_id);
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_array_access_empty_vec() {
        let empty_vec: Vec<i32> = vec![];
        
        // Test safe_first on empty vec
        assert_eq!(empty_vec.safe_first("test empty vec"), None);
        
        // Test safe_get on empty vec
        assert_eq!(empty_vec.safe_get(0, "test empty vec index 0"), None);
        assert_eq!(empty_vec.safe_get(10, "test empty vec index 10"), None);
    }
    
    #[test]
    fn test_safe_array_access_single_element() {
        let single_vec = vec![42];
        
        // Test safe_first on single element vec
        assert_eq!(single_vec.safe_first("test single vec"), Some(&42));
        
        // Test safe_get on single element vec
        assert_eq!(single_vec.safe_get(0, "test single vec index 0"), Some(&42));
        assert_eq!(single_vec.safe_get(1, "test single vec index 1"), None);
    }
    
    #[test]
    fn test_safe_array_access_multiple_elements() {
        let multi_vec = vec![1, 2, 3, 4, 5];
        
        // Test safe_first on multi element vec
        assert_eq!(multi_vec.safe_first("test multi vec"), Some(&1));
        
        // Test safe_get on multi element vec
        assert_eq!(multi_vec.safe_get(0, "test multi vec index 0"), Some(&1));
        assert_eq!(multi_vec.safe_get(2, "test multi vec index 2"), Some(&3));
        assert_eq!(multi_vec.safe_get(4, "test multi vec index 4"), Some(&5));
        assert_eq!(multi_vec.safe_get(5, "test multi vec index 5"), None);
        assert_eq!(multi_vec.safe_get(100, "test multi vec index 100"), None);
    }
    
    #[test]
    fn test_safe_array_access_with_strings() {
        let string_vec = vec!["hello".to_string(), "world".to_string()];
        
        // Test with String type
        assert_eq!(
            string_vec.safe_first("test string vec"),
            Some(&"hello".to_string())
        );
        assert_eq!(
            string_vec.safe_get(1, "test string vec index 1"),
            Some(&"world".to_string())
        );
    }
    
    #[test]
    fn test_safe_array_access_with_structs() {
        #[derive(Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }
        
        let struct_vec = vec![
            TestStruct { value: 1 },
            TestStruct { value: 2 },
        ];
        
        // Test with custom struct
        assert_eq!(
            struct_vec.safe_first("test struct vec"),
            Some(&TestStruct { value: 1 })
        );
        assert_eq!(
            struct_vec.safe_get(1, "test struct vec index 1"),
            Some(&TestStruct { value: 2 })
        );
        assert_eq!(
            struct_vec.safe_get(2, "test struct vec index 2"),
            None
        );
    }
    
    #[test]
    fn test_safe_array_access_boundary_conditions() {
        let vec = vec![10, 20, 30];
        
        // Test boundary conditions
        assert_eq!(vec.safe_get(0, "boundary test first"), Some(&10));
        assert_eq!(vec.safe_get(2, "boundary test last"), Some(&30));
        assert_eq!(vec.safe_get(3, "boundary test beyond"), None);
        
        // Test with large index
        assert_eq!(vec.safe_get(usize::MAX, "boundary test max"), None);
    }
}