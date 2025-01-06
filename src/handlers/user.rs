use actix_web::{HttpResponse, cookie::Cookie};
use serde::Deserialize;
use anyhow_response_error::AnyhowError;
use anyhow::{Context, Result};
use actix_multipart::form::{json::Json as MpJson, MultipartForm};
use serde_json::json;
use sqlx::SqlitePool;
use bcrypt::hash;
use uuid::Uuid;
use sqlx::FromRow;


#[derive(Deserialize)]
struct FormDataJson {
    email: String,
    password: String,
    push_endpoint: String,
    push_p256dh: String,
    push_auth: String,
}

#[derive(MultipartForm)]
pub struct InputData {
    json: MpJson<FormDataJson>,
}

#[derive(FromRow, Debug)]
pub struct SubscriptionRecord {
    pub push_endpoint: String,
    pub push_p256dh: String,
    pub push_auth: String,
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


pub async fn register_account(
    pool: &SqlitePool,
    user_id: &str,
    email: &str,
    password: &str,
    push_endpoint: &str,
    push_p256dh: &str,
    push_auth: &str,
) -> Result<HttpResponse, anyhow::Error> {
    print!("register_account");

    let hashed_password = hash(password, 12).map_err(|e| {
        sqlx::Error::Protocol(format!("Password hashing failed: {:?}", e).into())
    })?;

    let result = sqlx::query(
        "INSERT INTO accounts (user_id, email, password, push_endpoint, push_p256dh, push_auth) 
        VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(email)
    .bind(&hashed_password)
    .bind(push_endpoint)
    .bind(push_p256dh)
    .bind(push_auth)
    .execute(pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 1 => {
            println!("Insert successful");

            let record = sqlx::query_as::<_, SubscriptionRecord>(
                "SELECT email, push_endpoint, push_p256dh, push_auth FROM accounts WHERE user_id = ?"
            )
            .bind(user_id)
            .fetch_one(pool)
            .await
            .context("Failed to fetch inserted record")?;

            println!("Inserted record: {:?}", record);

            let mut response = HttpResponse::Ok();
            response.cookie(Cookie::new("user_id", user_id.to_string()));
            Ok(response.finish())
        }
        Ok(_) => {
            println!("Insert failed");
            Ok(HttpResponse::InternalServerError().finish())
        }
        Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("accounts_email_key") => {
            println!("Email already exists");
            Ok(HttpResponse::Conflict().body("Email already exists"))
        }
        Err(e) => {
            println!("Failed to execute query: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn get_subscription_by_mailadress(
    pool: &SqlitePool,
    mail: &str,
) -> Result<SubscriptionRecord, sqlx::Error> {
    let record = sqlx::query_as::<_, SubscriptionRecord>(
        "SELECT push_endpoint, push_p256dh, push_auth FROM accounts WHERE email = ?"
    )
    .bind(mail)
    .fetch_one(pool)
    .await
    .context("Failed to send push notification").unwrap();
    Ok(record)
}

async fn send_push_notification(
    endpoint : String,
    p256dh : String,
    auth : String,
    title : String,
    message : String,
) {
    println!("{}", endpoint);
    let nextjs_api_url = "https://localhost:3000/api/web-push/send";

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "subscription":{
            "endpoint": endpoint.clone(),
            "keys":{
                "p256dh": p256dh.clone(),
                "auth": auth.clone(),
            },
        },
        "title": title,
        "message": message,
        
    });
    let responce = client.post(nextjs_api_url)
        .json(&payload)
        .send()
        .await;
    match responce {
        Ok(responce) => {
            match responce.status().as_u16() {
                200 => println!("Push notification sent"),
                _ => println!("Failed to send push notification"),
            }
        }
        Err(e) => {
            println!("Failed to send push notification: {:?}", e);
        }
    }
}

pub async fn handler_user_create(
    MultipartForm(form): MultipartForm<InputData>,
) -> Result<HttpResponse, AnyhowError> {
    println!("handler_user_create");
    let user_id = Uuid::new_v4().to_string();
    let email = &form.json.email;
    let password = &form.json.password;
    let push_endpoint = &form.json.push_endpoint;
    let push_p256dh = &form.json.push_p256dh;
    let push_auth = &form.json.push_auth;

    let pool = crate::database::get_pool().await.map_err(|e| {
        println!("Failed to get database pool: {:?}", e);
        anyhow::Error::msg(e)
    })?;

    let title = "Welcome!".to_string();
    let message = "Thank you for registering!".to_string();

    match register_account(&pool, &user_id, email, password, push_endpoint, push_p256dh, push_auth).await {
        Ok(response) => {
            // データベースエラーがない場合のみ通知を送信
            if response.status().is_success() {
                send_push_notification(push_endpoint.clone(), push_p256dh.clone(), push_auth.clone(), title, message).await;
            }
            Ok(response)
        }
        Err(e) => {
            if let Some(db_err) = e.downcast_ref::<sqlx::Error>() {
                if let sqlx::Error::Database(db_err) = db_err {
                    if db_err.constraint() == Some("accounts_email_key") {
                        println!("Email already exists: {:?}", e);
                        return Ok(HttpResponse::Conflict().json(json!({
                            "message": "Email already exists",
                            "redirect": "/login"
                        })));
                    }
                }
            }
            println!("Failed to register account: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to execute query: {:?}", e)
            })));
        }
    }
}

pub async fn handler_user_login(
    MultipartForm(form): MultipartForm<InputData>,
) -> Result<HttpResponse, AnyhowError> {
    println!("handler_user_login");
    let email = &form.json.email;
    let password = &form.json.password;

    let pool = crate::database::get_pool().await.map_err(anyhow::Error::msg)?;

    let record = sqlx::query_as::<_, LoginFormDataJson>(
        "SELECT user_id, email, password, push_endpoint, push_p256dh, push_auth FROM accounts WHERE email = ?"
    )
    .bind(email)
    .fetch_one(&pool)
    .await
    .map_err(anyhow::Error::msg)?;

    let hashed_password = record.password;
    if bcrypt::verify(password, &hashed_password).map_err(|e| {
        sqlx::Error::Protocol(format!("Password verification failed: {:?}", e).into())
    }).map_err(anyhow::Error::msg)? {
        let mut response = HttpResponse::Ok();
        response.cookie(Cookie::new("user_id", record.user_id));
        Ok(response.finish())
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
