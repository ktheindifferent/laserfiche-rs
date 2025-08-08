// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.

use crate::laserfiche::{
    LFApiServer, LFAPIError, AuthOrError, Auth as AsyncAuth,
    Fields, Field, FieldValue, EntryOrError, ImportResultOrError,
    Entry, Entries, EntriesOrError, MetadataResult, MetadataResultOrError,
    ImportResult, BitsOrError, LFObject, DeletedObject
};

use serde_json::json;
use std::io::Cursor;
use error_chain::error_chain;
use std::time::{SystemTime, UNIX_EPOCH};

error_chain! {
    foreign_links {
        HttpRequest(reqwest::Error);
        IOError(std::io::Error);
    }
}

/// Blocking version of Auth - reuses types from async module
pub type Auth = AsyncAuth;

impl Auth {
    /// Synchronous authentication
    pub fn new_blocking(api_server: LFApiServer, username: String, password: String) -> Result<AuthOrError> {
        Self::authenticate_blocking(api_server, username, password)
    }

    /// Synchronous token refresh
    pub fn refresh_blocking(&self) -> Result<AuthOrError> {
        Self::authenticate_blocking(
            self.api_server.clone(),
            self.username.clone(),
            self.password.clone()
        )
    }

    fn authenticate_blocking(api_server: LFApiServer, username: String, password: String) -> Result<AuthOrError> {
        let token_url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Token",
            api_server.address,
            api_server.repository
        );
        
        let auth_params = vec![
            ("grant_type", "password"),
            ("username", username.as_str()),
            ("password", password.as_str()),
        ];
        
        let response = reqwest::blocking::Client::new()
            .post(token_url)
            .form(&auth_params)
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(AuthOrError::LFAPIError(error));
        }

        let mut auth = response.json::<Self>()?;
        auth.username = username;
        auth.password = password;
        auth.api_server = api_server;
        auth.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        Ok(AuthOrError::Auth(auth))
    }
}

/// Blocking API methods for Entry operations
impl Entry {
    /// Blocking version of import
    pub fn import_blocking(
        api_server: LFApiServer,
        auth: Auth,
        file_path: String,
        file_name: String,
        root_id: i64
    ) -> Result<ImportResultOrError> {
        let file_content = std::fs::read(&file_path)?;
        
        let file_part = reqwest::blocking::multipart::Part::bytes(file_content)
            .file_name(file_name.clone())
            .mime_str("image/png")
            .unwrap();

        let request_part = reqwest::blocking::multipart::Part::text("{}")
            .mime_str("application/json")
            .unwrap();

        let form = reqwest::blocking::multipart::Form::new()
            .part("electronicDocument", file_part)
            .part("request", request_part);

        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/{}?autoRename=true",
            api_server.address,
            api_server.repository,
            root_id,
            file_name
        );

        let response = reqwest::blocking::Client::new()
            .post(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .multipart(form)
            .send()?;

        if response.status() != reqwest::StatusCode::CREATED {
            let error = response.json::<LFAPIError>()?;
            return Ok(ImportResultOrError::LFAPIError(error));
        }

        let result = response.json::<ImportResult>()?;
        Ok(ImportResultOrError::ImportResult(result))
    }

    /// Blocking version of get
    pub fn get_blocking(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64
    ) -> Result<EntryOrError> {
        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}",
            api_server.address,
            api_server.repository,
            root_id
        );
        
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(EntryOrError::LFAPIError(error));
        }

        let entry = response.json::<Self>()?;
        Ok(EntryOrError::Entry(entry))
    }

    /// Blocking version of list
    pub fn list_blocking(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64
    ) -> Result<EntriesOrError> {
        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Folder/children",
            api_server.address,
            api_server.repository,
            root_id
        );
        
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(EntriesOrError::LFAPIError(error));
        }

        let entries = response.json::<Entries>()?;
        Ok(EntriesOrError::Entries(entries))
    }

    /// Blocking version of export
    pub fn export_blocking(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        file_path: &str
    ) -> Result<BitsOrError> {
        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Document/edoc",
            api_server.address,
            api_server.repository,
            entry_id
        );
        
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(BitsOrError::LFAPIError(error));
        }

        let bytes = response.bytes()?;
        let mut file = std::fs::File::create(file_path)?;
        let mut cursor = Cursor::new(&bytes);
        std::io::copy(&mut cursor, &mut file)?;
        
        Ok(BitsOrError::Bits(bytes.to_vec()))
    }

    /// Blocking version of get_metadata
    pub fn get_metadata_blocking(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<MetadataResultOrError> {
        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields",
            api_server.address,
            api_server.repository,
            entry_id
        );
        
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(MetadataResultOrError::LFAPIError(error));
        }

        let metadata = response.json::<MetadataResult>()?;
        Ok(MetadataResultOrError::Metadata(metadata))
    }

    /// Blocking version of update_metadata
    pub fn update_metadata_blocking(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        metadata: serde_json::Value
    ) -> Result<MetadataResultOrError> {
        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields",
            api_server.address,
            api_server.repository,
            entry_id
        );
        
        let response = reqwest::blocking::Client::new()
            .put(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&metadata)
            .send()?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>()?;
            return Ok(MetadataResultOrError::LFAPIError(error));
        }

        let metadata = response.json::<MetadataResult>()?;
        Ok(MetadataResultOrError::Metadata(metadata))
    }

    /// Blocking version of delete
    pub fn delete_blocking(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64,
        comment: String
    ) -> Result<LFObject> {
        let params = json!({
            "auditReasonId": 0,
            "comment": comment
        });

        let url = format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}",
            api_server.address,
            api_server.repository,
            root_id
        );
        
        let response = reqwest::blocking::Client::new()
            .delete(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send()?;

        if response.status() != reqwest::StatusCode::CREATED {
            let error = response.json::<LFAPIError>()?;
            return Ok(LFObject::LFAPIError(error));
        }

        let deleted = response.json::<DeletedObject>()?;
        Ok(LFObject::DeletedObject(deleted))
    }
}