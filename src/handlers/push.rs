use actix_web::HttpResponse;
use std::result::Result;
use serde::Deserialize;
use actix_multipart::form::{json::Json as MpJson, MultipartForm};
use anyhow_response_error::AnyhowError;
use crate::handlers::user::get_subscription_by_mailadress;
use std::env;

#[derive(Deserialize)]
struct FormDataJson {
    email: String,
    title: String,
    message: String,
}

#[derive(MultipartForm)]
pub struct 
InputData {
    json: MpJson<FormDataJson>,
}

pub async fn handler_push(input: MultipartForm<InputData>) -> Result<HttpResponse, AnyhowError> {
    let base_url = env::var("NEXTJS_URL").unwrap_or("http://localhost:3000".to_string());
    let nextjs_api_url = format!("{}/api/web-push/send", base_url);
    let pool = crate::database::get_pool().await.map_err(anyhow::Error::msg)?;
    let record = get_subscription_by_mailadress(&pool,&input.json.email).await.map_err(anyhow::Error::msg)?;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "subscription":{
            "endpoint": record.push_endpoint.clone(),
            "keys":{
                "p256dh": record.push_p256dh.clone(),
                "auth": record.push_auth.clone(),
            },
        },
        "title": &input.json.title,
        "message": &input.json.message,
    });
    let responce = client.post(nextjs_api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            anyhow::Error::new(e)
        })?;
    match responce.status().as_u16() {
        200 => Ok(HttpResponse::Ok().finish()),
        _ => Ok(HttpResponse::InternalServerError().finish()),
    }
}
