use actix_multipart::form::{json::Json as MpJson, MultipartForm};
use actix_web::HttpResponse;
use anyhow_response_error::AnyhowError;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct JsonData {
    test: String,
}

#[derive(MultipartForm)]
pub struct InputData {
    json: MpJson<JsonData>,
}

pub async fn handler_test(
    MultipartForm(form): MultipartForm<InputData>,
) -> Result<HttpResponse, AnyhowError> {

    let message = format!("Test: {}", form.json.test);
    let json = json!({ "message": message });

    Ok(HttpResponse::Ok().body(json.to_string()))
}
