use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    App, HttpResponse, HttpServer,
};

use actix_web_oauth2::{AuthRequest, OAuth};
use oauth2::TokenResponse;
use tokio::sync::Mutex;

struct AppState {
    oauth: Mutex<OAuth>,
}

#[get("/")]
async fn index() -> HttpResponse {
    let link = "login";

    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            <a href="/{}">{}</a>
        </body>
    </html>"#,
        link, link
    );
    HttpResponse::Ok().body(html)
}

#[get("/login")]
async fn login(data: Data<AppState>) -> HttpResponse {
    let auth_url = data.oauth.lock().await.auth_url();
    HttpResponse::Found()
        .append_header((header::LOCATION, auth_url.to_string()))
        .finish()
}

#[get("/logout")]
async fn logout() -> HttpResponse {
    HttpResponse::Found()
        .append_header((header::LOCATION, "/"))
        .finish()
}

#[get("/auth")]
async fn auth(data: Data<AppState>, Query(params): Query<AuthRequest>) -> HttpResponse {
    let (state, token) = data
        .oauth
        .lock()
        .await
        .auth(params)
        .await
        .expect("Failed to get token");

    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            Google returned the following state:
            <pre>{}</pre>
            Google returned the following token:
            <pre>{:?}</pre>
        </body>
    </html>"#,
        state.secret(),
        token
    );
    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    println!("Visit the following URL in your browser: http://127.0.0.1:5000");
    Ok(HttpServer::new(move || {
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable.");
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");

        let oauth = OAuth::new(google_client_id, google_client_secret).into();

        App::new()
            .app_data(Data::new(AppState { oauth }))
            .service(index)
            .service(login)
            .service(logout)
            .service(auth)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await?)
}
