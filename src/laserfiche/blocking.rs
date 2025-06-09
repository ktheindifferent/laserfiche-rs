// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.
use serde_json::json;

use serde::{Serialize, Deserialize};
use std::convert::TryInto;
use std::io::Cursor;
use serde_json::Map;
use error_chain::error_chain;

use std::time::{SystemTime, UNIX_EPOCH};
error_chain! {
    foreign_links {
        HttpRequest(reqwest::Error);
        IOError(std::io::Error);
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
    pub fn new(api_server: LFApiServer, username: String, password: String) -> Result<AuthOrError> {

        let mut params = vec![];
        params.push(("grant_type", "password"));
        params.push(("username", username.as_str()));
        params.push(("password", password.as_str()));
        

        let request = reqwest::blocking::Client::new()
        .post(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Token", api_server.address, api_server.repository))
        .form(&params)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
            
                    return Ok(AuthOrError::LFAPIError(json));
                }

                let mut json = req.json::<Self>()?;
                json.username = username;
                json.password = password;
                json.api_server = api_server;
                json.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            
            
                return Ok(AuthOrError::Auth(json));
            },
            Err(err) => Err(err.into())
        }

    }
    pub fn refresh(&self) -> Result<AuthOrError> {

        // if time_now - self.timestamp >= self.expires_in


        let mut params = vec![];
        params.push(("grant_type", "password"));
        params.push(("username", self.username.as_str()));
        params.push(("password", self.password.as_str()));

        let request = reqwest::blocking::Client::new()
        .post(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Token", self.api_server.address, self.api_server.repository))
        .form(&params)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
            
                    return Ok(AuthOrError::LFAPIError(json));
                }

                let mut json = req.json::<Self>()?;

                json.username = self.username.clone();
                json.password = self.password.clone();
                json.api_server = self.api_server.clone();
                json.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            
                return Ok(AuthOrError::Auth(json));
            },
            Err(err) => Err(err.into())
        }

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
impl Entry {

    pub fn import(api_server: LFApiServer, auth: Auth, file_path: String, file_name: String, root_id: i64) -> Result<ImportResultOrError> {

        let file = std::fs::read(file_path.as_str()).unwrap();
        let file_part = reqwest::blocking::multipart::Part::bytes(file)
        .file_name(file_name.clone())
        .mime_str("image/png")
        .unwrap();


        let file_request_part = reqwest::blocking::multipart::Part::text("{}")
        .mime_str("application/json")
        .unwrap();

        let form = reqwest::blocking::multipart::Form::new().part("electronicDocument", file_part).part("request", file_request_part);


        let request = reqwest::blocking::Client::new()
        .post(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/{}?autoRename=true", api_server.address, api_server.repository, root_id, file_name))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .multipart(form)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::CREATED{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(ImportResultOrError::LFAPIError(json));
                }

                let json = req.json::<ImportResult>()?;
            
                return Ok(ImportResultOrError::ImportResult(json));
            },
            Err(err) => Err(err.into())
        }

    }

    pub fn new_path(api_server: LFApiServer, auth: Auth, folder_name: String, volume_name: String, root_id: i64) -> Result<EntryOrError> {

        let params = NewEntry {
            entry_type: "Folder".to_string(),
            name: folder_name,
            volume_name: volume_name,
        };

        let request = reqwest::blocking::Client::new()
        .post(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Folder/children", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .json(&params)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::CREATED{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>()?;
            
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }

    }


    pub fn update_metadata(api_server: LFApiServer, auth: Auth, entry_id: i64, metadata: serde_json::Value) -> Result<MetadataResultOrError> {



        let request = reqwest::blocking::Client::new()
        .put(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields", api_server.address, api_server.repository, entry_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .json(&metadata)
        .send();

        
        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(MetadataResultOrError::LFAPIError(json));
                }

                let json = req.json::<MetadataResult>()?;
            
                return Ok(MetadataResultOrError::Metadata(json));
            },
            Err(err) => Err(err.into())
        }

    }






    pub fn get_metadata(api_server: LFApiServer, auth: Auth, entry_id: i64) -> Result<MetadataResultOrError> {



        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields", api_server.address, api_server.repository, entry_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        
        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(MetadataResultOrError::LFAPIError(json));
                }

                let json = req.json::<MetadataResult>()?;
            
                return Ok(MetadataResultOrError::Metadata(json));
            },
            Err(err) => Err(err.into())
        }

    }



    pub fn edoc_head(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<EntryOrError> {


        let request = reqwest::blocking::Client::new()
        .head(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Document/edoc", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>()?;
            
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }

    }

    pub fn export(api_server: LFApiServer, auth: Auth, entry_id: i64, file_path: &str) -> Result<BitsOrError> {


        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Document/edoc", api_server.address, api_server.repository, entry_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {



                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(BitsOrError::LFAPIError(json));
                }

                let mut file = std::fs::File::create(file_path.clone())?;
                let mut content =  Cursor::new(req.bytes()?);
                std::io::copy(&mut content, &mut file)?;

                let data = std::fs::read(file_path)?;
            
                return Ok(BitsOrError::Bits(data));
            },
            Err(err) => Err(err.into())
        }

    }


    pub fn get(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<EntryOrError> {


        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(EntryOrError::LFAPIError(json));
                }

                let json = req.json::<Self>()?;
            
                return Ok(EntryOrError::Entry(json));
            },
            Err(err) => Err(err.into())
        }

    }


    pub fn get_field(api_server: LFApiServer, auth: Auth, root_id: i64, field_id: i64) -> Result<LFObject> {


        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields/{}", api_server.address, api_server.repository, root_id, field_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Field>()?;
            
                return Ok(LFObject::Field(json));
            },
            Err(err) => Err(err.into())
        }

    }

    pub fn get_fields(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<LFObject> {


        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/fields", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Fields>()?;
            
                return Ok(LFObject::Fields(json));
            },
            Err(err) => Err(err.into())
        }

    }

    // {
    //     "auditReasonId": 0,
    //     "comment": "string"
    // }
    pub fn delete(api_server: LFApiServer, auth: Auth, root_id: i64, comment: String) -> Result<LFObject> {
        let params = DestroyEntry {
            audit_reason_id: 0,
            comment: comment,
        };   

        let request = reqwest::blocking::Client::new()
        .delete(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .json(&params)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::CREATED{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<DeletedObject>()?;
            
                return Ok(LFObject::DeletedObject(json));
            },
            Err(err) => Err(err.into())
        }
    }

    // Move or rename entry
    pub fn patch(api_server: LFApiServer, auth: Auth, root_id: i64, parent_id: Option<i64>, new_name: Option<String>) -> Result<LFObject> {
        let params = PatchedEntry {
            parent_id: parent_id,
            name: new_name,
        };   

        let request = reqwest::blocking::Client::new()
        .patch(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .json(&params)
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(LFObject::LFAPIError(json));
                }

                let json = req.json::<Self>()?;
            
                return Ok(LFObject::Entry(json));
            },
            Err(err) => Err(err.into())
        }
    }



    pub fn list(api_server: LFApiServer, auth: Auth, root_id: i64) -> Result<EntriesOrError> {


        let request = reqwest::blocking::Client::new()
        .get(format!("https://{}/LFRepositoryAPI/v1/Repositories/{}/Entries/{}/Laserfiche.Repository.Folder/children", api_server.address, api_server.repository, root_id))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(EntriesOrError::LFAPIError(json));
                }

                let json = req.json::<Entries>()?;
            
                return Ok(EntriesOrError::Entries(json));
            },
            Err(err) => Err(err.into())
        }

    }



    pub fn list_custom(auth: Auth, url: String) -> Result<EntriesOrError> {
        let request = reqwest::blocking::Client::new()
        .get(format!("{}", url))
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .send();

        match request{
            Ok(req) => {

                if req.status() != reqwest::StatusCode::OK{
                    let json = req.json::<LFAPIError>()?;
                    return Ok(EntriesOrError::LFAPIError(json));
                }

                let json = req.json::<Entries>()?;
            
                return Ok(EntriesOrError::Entries(json));
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
