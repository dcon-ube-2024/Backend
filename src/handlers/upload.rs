use actix_web::HttpResponse;
use anyhow::{Context, Result};
use anyhow_response_error::{AnyhowError, anyhow_error};
use serde::Deserialize;
use actix_multipart::form::{json::Json as MpJson, tempfile::TempFile, MultipartForm};
use uuid::Uuid;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;
use std::time::Instant;
use std::io::Read;
use std::path::{Path, PathBuf};
use chrono::Local;

#[derive(Deserialize)]
struct JsonData {
    user_id: String,
}

#[derive(MultipartForm)]
pub struct InputData {
    file: TempFile,
    json: MpJson<JsonData>,
}

async fn save_file(input_file: TempFile,input_userid: String) -> Result<PathBuf, AnyhowError> {
    let extension = input_file.file_name.as_ref()
        .and_then(|s| s.rsplit('.').next())
        .unwrap_or("unknown");

    let sanitized_extension = extension.replace("\"", "");
    let file_name = Uuid::new_v4();
    let date = Local::now().format("%Y-%m-%d").to_string();

    let save_path = Path::new("./uploads")
        .join(input_userid)
        .join(date)
        .join(format!("{}.{}", file_name, sanitized_extension));

    if let Some(parent) = save_path.parent() {
        let create_dir_start = Instant::now();
        if !parent.exists() {
            create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory for path: {}", save_path.display()))?;
        }
        let create_dir_duration = create_dir_start.elapsed();
        println!("Directory creation took: {:?}", create_dir_duration);
    }

    let file_create_start = Instant::now();
    let mut file = File::create(&save_path).await
        .map_err(|e| anyhow_error(anyhow::anyhow!(
            "Error creating file at path: {}. Error: {:?}", save_path.display(), e
        )))?;
    let file_create_duration = file_create_start.elapsed();
    println!("File creation took: {:?}", file_create_duration);

    let chunk_size = 8 * 1024;
    let mut buffer = vec![0; chunk_size];
    let mut file_content = input_file.file;

    loop {
        let n = file_content.read(&mut buffer).unwrap();
        if n == 0 { break; }
        file.write_all(&buffer[..n]).await.unwrap();
    }

    Ok(save_path)
}

pub async fn handle_upload(MultipartForm(form): MultipartForm<InputData>) -> Result<HttpResponse, AnyhowError> {
    let start_time = Instant::now();

    let input_json = form.json;
    let input_file = form.file;

    if input_json.user_id.is_empty() || input_file.size == 0 {
        return Err(anyhow_error(anyhow::anyhow!("Input data is empty")));
    }

    let save_path = save_file(input_file,input_json.user_id.clone()).await?;

    println!("File saved at path: {}", save_path.display());

    let duration = start_time.elapsed();
    println!("Total processing time: {:?}", duration);

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}
