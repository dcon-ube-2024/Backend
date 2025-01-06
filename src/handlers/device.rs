use actix_web::HttpResponse;
use serde::Deserialize;
use anyhow_response_error::AnyhowError;
use anyhow::Result;
use actix_multipart::form::{json::Json as MpJson, MultipartForm};
use serde_json::json;
use sqlx::FromRow;

#[derive(Deserialize)]
struct FormDataJson {
    email: String,
    password: String,
}

#[derive(MultipartForm)]
pub struct InputData {
    json: MpJson<FormDataJson>,
}

#[derive(Deserialize, FromRow)]
struct LoginFormDataJson {
    user_id: String,
    email: String,
    password: String,
    push_endpoint: String,
    push_p256dh: String,
    push_auth: String,
}

pub async fn handler_user_loging_device(
    MultipartForm(form): MultipartForm<InputData>,
) -> Result<HttpResponse, AnyhowError> {
    println!("handler_user_loging_device");
    let email = &form.json.email;
    let password = &form.json.password;

    let pool = crate::database::get_pool().await.map_err(|e| {
        println!("Failed to get database pool: {:?}", e);
        anyhow::Error::msg(e)
    })?;

    let record = sqlx::query_as::<_, LoginFormDataJson>(
        "SELECT user_id, email, password, push_endpoint, push_p256dh, push_auth FROM accounts WHERE email = ?"
    )
    .bind(email)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        println!("Failed to fetch account: {:?}", e);
        anyhow::Error::msg(e)
    })?;

    let hashed_password = record.password;
    if bcrypt::verify(password, &hashed_password).map_err(|e| {
        sqlx::Error::Protocol(format!("Password verification failed: {:?}", e).into())
    }).map_err(anyhow::Error::msg)? {
        let response_body = json!({
            "user_id": record.user_id,
        });
        Ok(HttpResponse::Ok().json(response_body))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
