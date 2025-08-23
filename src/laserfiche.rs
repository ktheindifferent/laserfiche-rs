// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.
pub mod blocking;

use crate::validation;
use serde_json::json;

use serde::{Serialize, Deserialize};
use std::io::Cursor;
use error_chain::error_chain;
use std::time::{SystemTime, UNIX_EPOCH};
use std::convert::TryInto;

error_chain! {
    foreign_links {
        HttpRequest(reqwest::Error);
        IOError(std::io::Error);
        ValidationError(validation::Error);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LFApiServer {
    pub address: String,
    pub repository: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct LFAPIError {
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub title: Option<String>,
    pub status: Option<i64>,
    pub detail: Option<String>,
    pub instance: Option<String>,
    pub operation_id: Option<String>,
    pub error_source: Option<String>,
    pub error_code: Option<i64>,
    pub trace_id: Option<String>,
    pub additional_prop1: Option<String>,
    pub additional_prop2: Option<String>,
    pub additional_prop3: Option<String>,
}

pub enum AuthOrError {
    Auth(Auth),
    LFAPIError(LFAPIError),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Auth {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i64,
    #[serde(rename = "token_type")]
    pub token_type: String,
    #[serde(skip)]
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(skip)]
    pub timestamp: i64,
    #[serde(skip)]
    pub api_server: LFApiServer,
}
impl Auth {
    pub async fn new(api_server: LFApiServer, username: String, password: String) -> Result<AuthOrError> {
        Self::authenticate(api_server, username, password).await
    }

    pub async fn refresh(&self) -> Result<AuthOrError> {
        Self::authenticate(
            self.api_server.clone(),
            self.username.clone(),
            self.password.clone()
        ).await
    }

    async fn authenticate(api_server: LFApiServer, username: String, password: String) -> Result<AuthOrError> {
        // Validate server address and repository name
        let validated_address = validation::validate_server_address(&api_server.address)?;
        let validated_repository = validation::validate_repository_name(&api_server.repository)?;
        
        let validated_server = LFApiServer {
            address: validated_address,
            repository: validated_repository,
        };
        
        let token_url = Self::build_token_url(&validated_server);
        let auth_params = Self::build_auth_params(&username, &password);
        
        let response = reqwest::Client::new()
            .post(token_url)
            .form(&auth_params)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>().await?;
            return Ok(AuthOrError::LFAPIError(error));
        }

        let mut auth = response.json::<Self>().await?;
        auth.username = username;
        auth.password = password;
        auth.api_server = validated_server;
        auth.timestamp = Self::current_timestamp();
        
        Ok(AuthOrError::Auth(auth))
    }

    fn build_token_url(api_server: &LFApiServer) -> String {
        format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Token", 
            api_server.address, 
            api_server.repository)
    }

    fn build_auth_params<'a>(username: &'a str, password: &'a str) -> Vec<(&'static str, &'a str)> {
        vec![
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ]
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs()
            .try_into()
            .unwrap_or(i64::MAX)
    }
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub value: Vec<Field>,
    #[serde(rename = "@odata.nextLink")]
    pub odata_next_link: Option<String>,
    #[serde(rename = "@odata.count")]
    pub odata_count: Option<i64>,
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub field_name: String,
    pub values: Vec<FieldValue>,
    pub field_type: String,
    pub field_id: i64,
    pub is_multi_value: bool,
    pub is_required: bool,
    pub has_more_values: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FieldValue {
    pub additional_prop1: Option<String>,
    pub additional_prop2: Option<String>,
    pub additional_prop3: Option<String>,
}

pub enum EntryOrError {
    Entry(Entry),
    LFAPIError(LFAPIError),
}

pub enum ImportResultOrError {
    ImportResult(ImportResult),
    LFAPIError(LFAPIError),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct DestroyEntry {
    audit_reason_id: i64,
    comment: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct PatchedEntry {
    parent_id: Option<i64>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct NewEntry {
    entry_type: String,
    name: String,
    volume_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Entries {
    pub value: Vec<Entry>,
    #[serde(rename = "@odata.nextLink")]
    pub odata_next_link: Option<String>,
    #[serde(rename = "@odata.count")]
    pub odata_count: Option<i64>,
}




pub enum MetadataResultOrError {
    Metadata(MetadataResult),
    LFAPIError(LFAPIError),
}

pub enum BitsOrError {
    Bits(Vec<u8>),
    LFAPIError(LFAPIError),
}

pub enum EntriesOrError {
    Entries(Entries),
    LFAPIError(LFAPIError),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeletedObject {
    token: String,
}

pub enum LFObject {
    Fields(Fields),
    Field(Field),
    Entry(Entry),
    Entries(Entries),
    DeletedObject(DeletedObject),
    LFAPIError(LFAPIError),
}

/// Template information for an entry
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub field_count: i64,
}

pub enum TemplateOrError {
    Template(Template),
    LFAPIError(LFAPIError),
}

/// Tags associated with an entry
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    pub value: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_secure: bool,
    pub watermark_text: Option<String>,
}

pub enum TagsOrError {
    Tags(Tags),
    LFAPIError(LFAPIError),
}

/// Links associated with an entry
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub value: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub link_type: String,
    pub description: Option<String>,
}

pub enum LinksOrError {
    Links(Links),
    LFAPIError(LFAPIError),
}


/// Represents a Laserfiche repository entry (document or folder)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub full_path: String,
    pub folder_path: String,
    pub creator: String,
    pub creation_time: String,
    pub last_modified_time: String,
    pub entry_type: String,
    pub is_container: bool,
    pub is_leaf: bool,
    pub template_name: Option<String>,
    pub template_id: i64,
    pub template_field_names: Option<Vec<String>>,
    pub volume_name: String,
    pub row_number: i64,
    pub fields: Option<Vec<Field>>,
}
/// Helper functions for API operations
struct ApiHelper;

impl ApiHelper {
    fn build_entries_url(api_server: &LFApiServer, entry_id: i64) -> Result<String> {
        let validated_id = validation::validate_entry_id(entry_id)?;
        Ok(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}",
            api_server.address,
            api_server.repository,
            validated_id))
    }

    fn build_base_url(api_server: &LFApiServer) -> String {
        format!("https://{}/LFRepositoryAPI/v1/Repositories/{}",
            api_server.address,
            api_server.repository)
    }

    async fn execute_request<T: for<'de> Deserialize<'de>>(
        request: reqwest::RequestBuilder,
        auth_token: &str,
        expected_status: reqwest::StatusCode,
    ) -> Result<std::result::Result<T, LFAPIError>> {
        let response = request
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await?;

        if response.status() != expected_status {
            let error = response.json::<LFAPIError>().await?;
            return Ok(Err(error));
        }

        let result = response.json::<T>().await?;
        Ok(Ok(result))
    }
}

impl Entry {
    /// Import a document into Laserfiche repository
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `file_path` - Path to the file to import
    /// * `file_name` - Name for the document in repository
    /// * `root_id` - Parent folder ID
    pub async fn import(
        api_server: LFApiServer,
        auth: Auth,
        file_path: String,
        file_name: String,
        root_id: i64
    ) -> Result<ImportResultOrError> {
        // Validate inputs
        let validated_path = validation::validate_file_path(&file_path)?;
        let validated_name = validation::validate_file_name(&file_name)?;
        let validated_root_id = validation::validate_entry_id(root_id)?;
        
        let file_content = std::fs::read(&validated_path)?;
        
        // Validate file size
        validation::validate_file_size(file_content.len() as u64)?;
        
        let form = Self::build_import_form(file_content, &validated_name);
        let import_url = Self::build_import_url(&api_server, validated_root_id, &validated_name);
        
        let response = reqwest::Client::new()
            .post(import_url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .multipart(form)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::CREATED {
            let error = response.json::<LFAPIError>().await?;
            return Ok(ImportResultOrError::LFAPIError(error));
        }

        let result = response.json::<ImportResult>().await?;
        Ok(ImportResultOrError::ImportResult(result))
    }

    fn build_import_form(file_content: Vec<u8>, file_name: &str) -> reqwest::multipart::Form {
        // Detect MIME type from file extension
        let mime_type = Self::detect_mime_type(file_name);
        
        let file_part = reqwest::multipart::Part::bytes(file_content)
            .file_name(file_name.to_string())
            .mime_str(&mime_type)
            .unwrap_or_else(|_| reqwest::multipart::Part::bytes(vec![]));

        let request_part = reqwest::multipart::Part::text("{}")
            .mime_str("application/json")
            .unwrap_or_else(|_| reqwest::multipart::Part::text("{}"));

        reqwest::multipart::Form::new()
            .part("electronicDocument", file_part)
            .part("request", request_part)
    }

    fn build_import_url(api_server: &LFApiServer, root_id: i64, file_name: &str) -> String {
        format!(
            "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/{}?autoRename=true",
            api_server.address,
            api_server.repository,
            root_id,
            file_name
        )
    }
    
    /// Detect MIME type based on file extension
    fn detect_mime_type(file_name: &str) -> String {
        let extension = file_name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();
            
        match extension.as_str() {
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "tiff" | "tif" => "image/tiff",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "xml" => "application/xml",
            "json" => "application/json",
            _ => "application/octet-stream"
        }.to_string()
    }

    /// Create a new folder in the repository
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `folder_name` - Name for the new folder
    /// * `volume_name` - Volume name
    /// * `root_id` - Parent folder ID
    pub async fn new_path(
        api_server: LFApiServer,
        auth: Auth,
        folder_name: String,
        volume_name: String,
        root_id: i64
    ) -> Result<EntryOrError> {
        let params = NewEntry {
            entry_type: "Folder".to_string(),
            name: folder_name,
            volume_name,
        };

        let url = format!(
            "{}/Entries/{}/Laserfiche.Repository.Folder/children",
            ApiHelper::build_base_url(&api_server),
            root_id
        );

        let response = reqwest::Client::new()
            .post(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send()
            .await?;

        Self::handle_entry_response(response, reqwest::StatusCode::CREATED).await
    }

    async fn handle_entry_response(
        response: reqwest::Response,
        expected_status: reqwest::StatusCode
    ) -> Result<EntryOrError> {
        if response.status() != expected_status {
            let error = response.json::<LFAPIError>().await?;
            return Ok(EntryOrError::LFAPIError(error));
        }
        
        let entry = response.json::<Entry>().await?;
        Ok(EntryOrError::Entry(entry))
    }


    /// Update metadata/field values for an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID to update
    /// * `metadata` - JSON object containing field values
    pub async fn update_metadata(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        metadata: serde_json::Value
    ) -> Result<MetadataResultOrError> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(entry_id)?;
        let validated_metadata = validation::validate_metadata_json(&metadata)?;
        
        let url = format!("{}/fields", ApiHelper::build_entries_url(&api_server, validated_id)?);
        
        let response = reqwest::Client::new()
            .put(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&validated_metadata)
            .send()
            .await?;

        Self::handle_metadata_response(response).await
    }

    /// Get metadata/field values for an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    pub async fn get_metadata(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<MetadataResultOrError> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(entry_id)?;
        
        let url = format!("{}/fields", ApiHelper::build_entries_url(&api_server, validated_id)?);
        
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        Self::handle_metadata_response(response).await
    }

    async fn handle_metadata_response(
        response: reqwest::Response
    ) -> Result<MetadataResultOrError> {
        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>().await?;
            return Ok(MetadataResultOrError::LFAPIError(error));
        }
        
        let metadata = response.json::<MetadataResult>().await?;
        Ok(MetadataResultOrError::Metadata(metadata))
    }



    pub async fn edoc_head(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<EntryOrError> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(root_id)?;

        let request = reqwest::Client::new()
        .head(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Document/edoc", api_server.address, api_server.repository, validated_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send().await;

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>().await?;
            
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }

    }

    /// Export/download a document from the repository
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Document entry ID
    /// * `file_path` - Path to save the exported file
    pub async fn export(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        file_path: &str
    ) -> Result<BitsOrError> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(entry_id)?;
        let validated_path = validation::validate_file_path(file_path)?;
        
        let url = format!(
            "{}/Laserfiche.Repository.Document/edoc",
            ApiHelper::build_entries_url(&api_server, validated_id)?
        );
        
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>().await?;
            return Ok(BitsOrError::LFAPIError(error));
        }

        let bytes = response.bytes().await?;
        Self::save_to_file(&bytes, validated_path.to_str().ok_or("Invalid path")?)?;
        
        Ok(BitsOrError::Bits(bytes.to_vec()))
    }

    fn save_to_file(bytes: &[u8], file_path: &str) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        let mut cursor = Cursor::new(bytes);
        std::io::copy(&mut cursor, &mut file)?;
        Ok(())
    }

    /// Get entry information by ID
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `root_id` - Entry ID
    pub async fn get(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64
    ) -> Result<EntryOrError> {
        let validated_id = validation::validate_entry_id(root_id)?;
        let url = ApiHelper::build_entries_url(&api_server, validated_id)?;
        
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        Self::handle_entry_response(response, reqwest::StatusCode::OK).await
    }


    pub async fn get_field(api_server: LFApiServer, auth: Auth, root_id: i64, field_id: i64) -> Result<LFObject> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(root_id)?;
        let validated_field_id = validation::validate_entry_id(field_id)?;

        let request = reqwest::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields/{}", api_server.address, api_server.repository, validated_id, validated_field_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send().await;

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Field>().await?;
            
                return Ok(LFObject::Field(json));
            },
            Err(err) => Err(err.into())
        }

    }

    pub async fn get_fields(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<LFObject> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(root_id)?;

        let request = reqwest::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields", api_server.address, api_server.repository, validated_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send().await;

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Fields>().await?;
            
                return Ok(LFObject::Fields(json));
            },
            Err(err) => Err(err.into())
        }

    }

    /// Delete an entry from the repository
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `root_id` - Entry ID to delete
    /// * `comment` - Audit comment for deletion
    pub async fn delete(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64,
        comment: String
    ) -> Result<LFObject> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(root_id)?;
        
        let params = DestroyEntry {
            audit_reason_id: 0,
            comment,
        };

        let url = ApiHelper::build_entries_url(&api_server, validated_id)?;
        
        let response = reqwest::Client::new()
            .delete(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send()
            .await?;

        if response.status() != reqwest::StatusCode::CREATED {
            let error = response.json::<LFAPIError>().await?;
            return Ok(LFObject::LFAPIError(error));
        }

        let deleted = response.json::<DeletedObject>().await?;
        Ok(LFObject::DeletedObject(deleted))
    }

    /// Move or rename an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `root_id` - Entry ID to move/rename
    /// * `parent_id` - New parent folder ID (for moving)
    /// * `new_name` - New name (for renaming)
    pub async fn patch(api_server: LFApiServer, auth: Auth, root_id: i64, parent_id: Option<i64>, new_name: Option<String>) -> Result<LFObject> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(root_id)?;
        let validated_parent_id = if let Some(pid) = parent_id {
            Some(validation::validate_entry_id(pid)?)
        } else {
            None
        };
        let validated_name = if let Some(name) = &new_name {
            Some(validation::validate_file_name(name)?)
        } else {
            None
        };
        
        let params = PatchedEntry {
            parent_id: validated_parent_id,
            name: validated_name.clone(),
        };   

        let request = reqwest::Client::new()
        .patch(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}", api_server.address, api_server.repository, validated_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .json(&params)
        .send().await;

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Self>().await?;
            
                return Ok(LFObject::Entry(json));
            },
            Err(err) => Err(err.into())
        }
    }



    /// List child entries of a folder
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `root_id` - Folder entry ID
    pub async fn list(
        api_server: LFApiServer,
        auth: Auth,
        root_id: i64
    ) -> Result<EntriesOrError> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(root_id)?;
        
        let url = format!(
            "{}/Laserfiche.Repository.Folder/children",
            ApiHelper::build_entries_url(&api_server, validated_id)?
        );
        
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        Self::handle_entries_response(response).await
    }

    async fn handle_entries_response(
        response: reqwest::Response
    ) -> Result<EntriesOrError> {
        if response.status() != reqwest::StatusCode::OK {
            let error = response.json::<LFAPIError>().await?;
            return Ok(EntriesOrError::LFAPIError(error));
        }
        
        let entries = response.json::<Entries>().await?;
        Ok(EntriesOrError::Entries(entries))
    }


    pub async fn list_custom(auth: Auth, url: String) -> Result<EntriesOrError> {
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        Self::handle_entries_response(response).await
    }

    /// Search for entries using OData query parameters
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `search_query` - Search query string
    /// * `order_by` - Optional OData orderBy parameter (e.g., "name asc")
    /// * `select` - Optional OData select parameter for field filtering
    /// * `skip` - Optional number of entries to skip
    /// * `top` - Optional maximum number of entries to return
    pub async fn search(
        api_server: LFApiServer, 
        auth: Auth, 
        search_query: String,
        order_by: Option<String>,
        select: Option<String>,
        skip: Option<i32>,
        top: Option<i32>
    ) -> Result<EntriesOrError> {
        let url = Self::build_search_url(&api_server, &search_query, order_by, select, skip, top);
        
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?;

        Self::handle_entries_response(response).await
    }

    fn build_search_url(
        api_server: &LFApiServer,
        search_query: &str,
        order_by: Option<String>,
        select: Option<String>,
        skip: Option<i32>,
        top: Option<i32>
    ) -> String {
        let mut url = format!(
            "{}/Entries/Search?q={}",
            ApiHelper::build_base_url(api_server),
            urlencoding::encode(search_query)
        );

        if let Some(order) = order_by {
            url.push_str(&format!("&$orderby={}", urlencoding::encode(&order)));
        }
        if let Some(sel) = select {
            url.push_str(&format!("&$select={}", urlencoding::encode(&sel)));
        }
        if let Some(s) = skip {
            url.push_str(&format!("&$skip={}", s));
        }
        if let Some(t) = top {
            url.push_str(&format!("&$top={}", t));
        }

        url
    }

    /// Copy an entry to a new location
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID to copy
    /// * `target_folder_id` - Destination folder ID
    /// * `new_name` - Optional new name for the copy
    pub async fn copy(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        target_folder_id: i64,
        new_name: Option<String>
    ) -> Result<EntryOrError> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(entry_id)?;
        let validated_target_id = validation::validate_entry_id(target_folder_id)?;
        let validated_name = if let Some(name) = &new_name {
            Some(validation::validate_file_name(name)?)
        } else {
            None
        };
        
        let mut params = json!({
            "targetId": validated_target_id
        });
        
        if let Some(name) = validated_name {
            params["name"] = json!(name);
        }

        let request = reqwest::Client::new()
            .post(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Copy",
                api_server.address, 
                api_server.repository, 
                validated_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::CREATED {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>().await?;
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Get the template associated with an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    pub async fn get_template(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<TemplateOrError> {
        // Validate entry ID
        let validated_id = validation::validate_entry_id(entry_id)?;
        
        let request = reqwest::Client::new()
            .get(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/template",
                api_server.address, 
                api_server.repository, 
                validated_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(TemplateOrError::LFAPIError(json));
                }

                let json = req.json::<Template>().await?;
                return Ok(TemplateOrError::Template(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Assign a template to an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    /// * `template_name` - Name of the template to assign
    pub async fn set_template(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        template_name: String
    ) -> Result<EntryOrError> {
        // Validate inputs
        let validated_id = validation::validate_entry_id(entry_id)?;
        let validated_template_name = validation::validate_field_name(&template_name)?;
        
        let params = json!({
            "templateName": validated_template_name
        });

        let request = reqwest::Client::new()
            .put(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/template",
                api_server.address, 
                api_server.repository, 
                validated_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>().await?;
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Remove template from an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    pub async fn remove_template(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<EntryOrError> {
        let request = reqwest::Client::new()
            .delete(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/template",
                api_server.address, 
                api_server.repository, 
                entry_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>().await?;
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Get tags assigned to an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    pub async fn get_tags(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<TagsOrError> {
        let request = reqwest::Client::new()
            .get(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/tags",
                api_server.address, 
                api_server.repository, 
                entry_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(TagsOrError::LFAPIError(json));
                }

                let json = req.json::<Tags>().await?;
                return Ok(TagsOrError::Tags(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Assign tags to an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    /// * `tag_ids` - List of tag IDs to assign
    pub async fn set_tags(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64,
        tag_ids: Vec<i64>
    ) -> Result<TagsOrError> {
        let params = json!({
            "tags": tag_ids
        });

        let request = reqwest::Client::new()
            .put(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/tags",
                api_server.address, 
                api_server.repository, 
                entry_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .json(&params)
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(TagsOrError::LFAPIError(json));
                }

                let json = req.json::<Tags>().await?;
                return Ok(TagsOrError::Tags(json));
            },
            Err(err) => Err(err.into())
        }
    }

    /// Get links associated with an entry
    /// 
    /// # Arguments
    /// * `api_server` - API server configuration
    /// * `auth` - Authentication token
    /// * `entry_id` - Entry ID
    pub async fn get_links(
        api_server: LFApiServer,
        auth: Auth,
        entry_id: i64
    ) -> Result<LinksOrError> {
        let request = reqwest::Client::new()
            .get(format!(
                "https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/links",
                api_server.address, 
                api_server.repository, 
                entry_id
            ))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send().await;

        match request {
            Ok(req) => {
                if req.status() != reqwest::StatusCode::OK {
                    let json = req.json::<LFAPIError>().await?;
                    return Ok(LinksOrError::LFAPIError(json));
                }

                let json = req.json::<Links>().await?;
                return Ok(LinksOrError::Links(json));
            },
            Err(err) => Err(err.into())
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataValue {
    pub value: String,
    pub position: i64,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub operations: Operations,
    pub document_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operations {
    pub entry_create: EntryCreate,
    pub set_edoc: SetEdoc,
    pub set_template: Option<SetTemplate>,
    pub set_fields: Option<SetFields>,
    pub set_tags: Option<SetTags>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryCreate {
    pub entry_id: i64,
    pub exceptions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetEdoc {
    pub exceptions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTemplate {
    pub template: String,
    pub exceptions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFields {
    pub field_count: i64,
    pub exceptions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTags {
    pub assigned_tags: Vec<String>,
    pub exceptions: Vec<String>,
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataResult {
    pub value: Vec<MetadataResultValue>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataResultValue {
    pub field_name: String,
    pub field_type: String,
    pub group_id: Option<i64>,
    pub field_id: i64,
    pub is_multi_value: bool,
    pub is_required: bool,
    pub values: Vec<MetadataResultFieldValue>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataResultFieldValue {
    pub value: Option<String>,
    pub position: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_api_server() -> LFApiServer {
        LFApiServer {
            address: "test.laserfiche.com".to_string(),
            repository: "test-repo".to_string(),
        }
    }

    fn mock_auth() -> Auth {
        Auth {
            odata_context: "test-context".to_string(),
            access_token: "test-token-12345".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            username: "test-user".to_string(),
            password: "test-pass".to_string(),
            api_server: mock_api_server(),
            timestamp: 1234567890,
        }
    }

    #[test]
    fn test_lfapiserver_creation() {
        let server = LFApiServer {
            address: "example.laserfiche.com".to_string(),
            repository: "my-repo".to_string(),
        };
        assert_eq!(server.address, "example.laserfiche.com");
        assert_eq!(server.repository, "my-repo");
    }

    #[test]
    fn test_auth_struct_fields() {
        let auth = mock_auth();
        assert_eq!(auth.token_type, "Bearer");
        assert_eq!(auth.access_token, "test-token-12345");
        assert_eq!(auth.timestamp, 1234567890);
        assert_eq!(auth.username, "test-user");
        assert_eq!(auth.password, "test-pass");
        assert_eq!(auth.expires_in, 3600);
    }

    #[test]
    fn test_detect_mime_type() {
        assert_eq!(Entry::detect_mime_type("test.pdf"), "application/pdf");
        assert_eq!(Entry::detect_mime_type("test.jpg"), "image/jpeg");
        assert_eq!(Entry::detect_mime_type("test.jpeg"), "image/jpeg");
        assert_eq!(Entry::detect_mime_type("test.png"), "image/png");
        assert_eq!(Entry::detect_mime_type("test.gif"), "image/gif");
        assert_eq!(Entry::detect_mime_type("test.tif"), "image/tiff");
        assert_eq!(Entry::detect_mime_type("test.tiff"), "image/tiff");
        assert_eq!(Entry::detect_mime_type("test.txt"), "text/plain");
        assert_eq!(Entry::detect_mime_type("test.csv"), "text/csv");
        assert_eq!(Entry::detect_mime_type("test.xml"), "application/xml");
        assert_eq!(Entry::detect_mime_type("test.json"), "application/json");
        assert_eq!(Entry::detect_mime_type("test.doc"), "application/msword");
        assert_eq!(Entry::detect_mime_type("test.docx"), "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        assert_eq!(Entry::detect_mime_type("test.xls"), "application/vnd.ms-excel");
        assert_eq!(Entry::detect_mime_type("test.xlsx"), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        assert_eq!(Entry::detect_mime_type("test.ppt"), "application/vnd.ms-powerpoint");
        assert_eq!(Entry::detect_mime_type("test.pptx"), "application/vnd.openxmlformats-officedocument.presentationml.presentation");
        assert_eq!(Entry::detect_mime_type("test.unknown"), "application/octet-stream");
        assert_eq!(Entry::detect_mime_type("test"), "application/octet-stream");
    }

    #[test]
    fn test_detect_mime_type_case_insensitive() {
        assert_eq!(Entry::detect_mime_type("TEST.PDF"), "application/pdf");
        assert_eq!(Entry::detect_mime_type("Test.Pdf"), "application/pdf");
        assert_eq!(Entry::detect_mime_type("test.PDF"), "application/pdf");
    }

    #[test]
    fn test_entry_struct() {
        let entry = Entry {
            id: 123,
            name: "test-document.pdf".to_string(),
            parent_id: 1,
            full_path: "/root/test-document.pdf".to_string(),
            folder_path: "/root".to_string(),
            creator: "john.doe".to_string(),
            creation_time: "2024-01-01T00:00:00Z".to_string(),
            last_modified_time: "2024-01-02T00:00:00Z".to_string(),
            entry_type: "Document".to_string(),
            is_container: false,
            is_leaf: true,
            volume_name: "Volume1".to_string(),
            row_number: 1,
            ..Default::default()
        };

        assert_eq!(entry.id, 123);
        assert_eq!(entry.name, "test-document.pdf");
        assert_eq!(entry.parent_id, 1);
        assert!(!entry.is_container);
        assert!(entry.is_leaf);
    }

    #[test]
    fn test_entries_collection() {
        let entry1 = Entry {
            id: 1,
            name: "doc1.pdf".to_string(),
            ..Default::default()
        };
        
        let entry2 = Entry {
            id: 2,
            name: "doc2.pdf".to_string(),
            ..Default::default()
        };

        let entries = Entries {
            value: vec![entry1, entry2],
            odata_next_link: Some("https://api.laserfiche.com/next".to_string()),
            ..Default::default()
        };

        assert_eq!(entries.value.len(), 2);
        assert_eq!(entries.value[0].id, 1);
        assert_eq!(entries.value[1].id, 2);
        assert!(entries.odata_next_link.is_some());
    }

    #[test]
    fn test_patched_entry_struct() {
        // Test PatchedEntry instead of non-existent DeleteParameters
        let patched = PatchedEntry {
            parent_id: Some(10),
            name: Some("renamed-document.pdf".to_string()),
        };

        assert_eq!(patched.parent_id, Some(10));
        assert_eq!(patched.name, Some("renamed-document.pdf".to_string()));
    }

    #[test]
    fn test_patched_entry_serialization() {
        let patched = PatchedEntry {
            parent_id: Some(10),
            name: Some("renamed-document.pdf".to_string()),
        };

        let json = serde_json::to_string(&patched).unwrap();
        assert!(json.contains("\"parentId\":10"));
        assert!(json.contains("\"name\":\"renamed-document.pdf\""));
    }

    #[test]
    fn test_metadata_value() {
        let metadata = MetadataValue {
            value: "test-value".to_string(),
            position: 1,
        };

        assert_eq!(metadata.value, "test-value");
        assert_eq!(metadata.position, 1);
    }

    #[test]
    fn test_import_result() {
        let import_result = ImportResult {
            operations: Operations {
                entry_create: EntryCreate {
                    entry_id: 123,
                    exceptions: vec![],
                },
                set_edoc: SetEdoc {
                    exceptions: vec![],
                },
                set_template: None,
                set_fields: None,
                set_tags: None,
            },
            document_link: "https://api.laserfiche.com/entries/123".to_string(),
        };

        assert_eq!(import_result.operations.entry_create.entry_id, 123);
        assert!(import_result.operations.entry_create.exceptions.is_empty());
        assert_eq!(import_result.document_link, "https://api.laserfiche.com/entries/123");
    }

    #[test]
    fn test_lfapi_error() {
        let error = LFAPIError {
            type_field: Some("NotFoundError".to_string()),
            title: Some("Not Found".to_string()),
            status: Some(404),
            detail: Some("The requested entry does not exist".to_string()),
            instance: Some("/api/entries/999".to_string()),
            operation_id: Some("op-123".to_string()),
            error_source: Some("Repository".to_string()),
            error_code: Some(1001),
            trace_id: Some("trace-123".to_string()),
            additional_prop1: None,
            additional_prop2: None,
            additional_prop3: None,
        };

        assert_eq!(error.status, Some(404));
        assert_eq!(error.title, Some("Not Found".to_string()));
        assert_eq!(error.error_code, Some(1001));
    }

    #[test]
    fn test_auth_or_error_enum() {
        let auth = mock_auth();
        let auth_result = AuthOrError::Auth(auth.clone());
        
        match auth_result {
            AuthOrError::Auth(a) => assert_eq!(a.access_token, "test-token-12345"),
            AuthOrError::LFAPIError(_) => panic!("Expected Auth variant"),
        }

        let error = LFAPIError {
            status: Some(401),
            title: Some("Unauthorized".to_string()),
            ..Default::default()
        };
        let error_result = AuthOrError::LFAPIError(error);
        
        match error_result {
            AuthOrError::Auth(_) => panic!("Expected LFAPIError variant"),
            AuthOrError::LFAPIError(e) => assert_eq!(e.status, Some(401)),
        }
    }

    #[test]
    fn test_entry_or_error_enum() {
        let entry = Entry {
            id: 123,
            name: "test.pdf".to_string(),
            ..Default::default()
        };
        let entry_result = EntryOrError::Entry(entry);
        
        match entry_result {
            EntryOrError::Entry(e) => assert_eq!(e.id, 123),
            EntryOrError::LFAPIError(_) => panic!("Expected Entry variant"),
        }
    }

    #[test]
    fn test_entries_or_error_enum() {
        let entries = Entries {
            value: vec![],
            ..Default::default()
        };
        let entries_result = EntriesOrError::Entries(entries);
        
        match entries_result {
            EntriesOrError::Entries(e) => assert_eq!(e.value.len(), 0),
            EntriesOrError::LFAPIError(_) => panic!("Expected Entries variant"),
        }
    }

    #[test]
    fn test_import_result_or_error_enum() {
        let import = ImportResult {
            operations: Operations {
                entry_create: EntryCreate {
                    entry_id: 456,
                    exceptions: vec![],
                },
                set_edoc: SetEdoc {
                    exceptions: vec![],
                },
                set_template: None,
                set_fields: None,
                set_tags: None,
            },
            document_link: "https://test.com/456".to_string(),
        };
        let import_result = ImportResultOrError::ImportResult(import);
        
        match import_result {
            ImportResultOrError::ImportResult(i) => {
                assert_eq!(i.operations.entry_create.entry_id, 456)
            },
            ImportResultOrError::LFAPIError(_) => panic!("Expected ImportResult variant"),
        }
    }

    #[test]
    fn test_timestamp_year_2038_boundary() {
        // Year 2038 problem occurs at 2^31 - 1 seconds (January 19, 2038 03:14:07 UTC)
        let year_2038_timestamp: u64 = 2_147_483_647;
        let result: std::result::Result<i64, _> = year_2038_timestamp.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2_147_483_647);
        
        // One second after the 2038 boundary (still within i64 range)
        let after_2038: u64 = 2_147_483_648;
        let result: std::result::Result<i64, _> = after_2038.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2_147_483_648);
    }

    #[test]
    fn test_timestamp_max_i64_value() {
        // Test at exactly i64::MAX
        let max_i64_as_u64: u64 = i64::MAX as u64;
        let result: std::result::Result<i64, _> = max_i64_as_u64.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), i64::MAX);
        
        // Test overflow scenario - one more than i64::MAX
        let overflow: u64 = (i64::MAX as u64) + 1;
        let result: std::result::Result<i64, _> = overflow.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_overflow_handling() {
        // Test that our implementation handles overflow gracefully
        let max_u64 = u64::MAX;
        let result: i64 = max_u64.try_into().unwrap_or(i64::MAX);
        assert_eq!(result, i64::MAX);
        
        // Test with a value that would overflow
        let overflow_value: u64 = (i64::MAX as u64) + 1000;
        let result: i64 = overflow_value.try_into().unwrap_or(i64::MAX);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    fn test_current_timestamp_safe_conversion() {
        // Test that current_timestamp returns a valid i64
        let timestamp = Auth::current_timestamp();
        assert!(timestamp > 0);
        assert!(timestamp <= i64::MAX);
        
        // Verify it's approximately the current time (within reasonable bounds)
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check that the timestamp is reasonable (between year 2020 and 2100)
        let year_2020: i64 = 1577836800; // January 1, 2020
        let year_2100: i64 = 4102444800; // January 1, 2100
        assert!(timestamp >= year_2020);
        assert!(timestamp <= year_2100);
        
        // The current timestamp should be close to now
        assert!((timestamp as u64) <= now_secs + 1);
    }

    #[test]
    fn test_future_dates_handling() {
        // Test with year 2050 timestamp
        let year_2050: u64 = 2524608000; // January 1, 2050
        let result: std::result::Result<i64, _> = year_2050.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2524608000);
        
        // Test with year 2100 timestamp
        let year_2100: u64 = 4102444800; // January 1, 2100
        let result: std::result::Result<i64, _> = year_2100.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4102444800);
        
        // Test with year 2200 timestamp (well within i64 range)
        let year_2200: u64 = 7258118400; // January 1, 2200
        let result: std::result::Result<i64, _> = year_2200.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 7258118400);
    }

    #[test]
    fn test_edge_case_zero_timestamp() {
        // Test Unix epoch (0)
        let epoch: u64 = 0;
        let result: std::result::Result<i64, _> = epoch.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_auth_timestamp_field() {
        // Create an Auth instance and verify timestamp is set correctly
        let mut auth = mock_auth();
        
        // Set timestamp to a known value
        auth.timestamp = 1234567890;
        assert_eq!(auth.timestamp, 1234567890);
        
        // Test setting to max value
        auth.timestamp = i64::MAX;
        assert_eq!(auth.timestamp, i64::MAX);
        
        // Verify current_timestamp is within valid range
        auth.timestamp = Auth::current_timestamp();
        assert!(auth.timestamp > 0);
        assert!(auth.timestamp <= i64::MAX);
    }
}
