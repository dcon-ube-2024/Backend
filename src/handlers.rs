use actix_web::web;

mod upload;
mod user;
mod push;
mod test;
mod device;

// ルーティング設定を一括管理
pub fn configure(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig{
    cfg
        .route("/api/upload", web::post().to(upload::handle_upload))
        .route("/api/account_create", web::post().to(user::handler_user_create))
        .route("/api/test", web::post().to(test::handler_test))
        .route("/api/push", web::post().to(push::handler_push))
        .route("/api/login", web::post().to(user::handler_user_login))
        .route("/api/login_device", web::post().to(device::handler_user_loging_device))
    }
