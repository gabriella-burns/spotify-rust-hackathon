use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::sync::Mutex;
use std::error::Error;

#[derive(Clone)]
struct AppState {
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthResponse {
    access_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    popularity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopTracksResponse {
    tracks: Vec<Track>,
}

async fn get_access_token(client_id: &str, client_secret: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await?;

    let auth_response: AuthResponse = response.json().await?;
    Ok(auth_response.access_token)
}

async fn get_artist_top_tracks(access_token: &str, artist_id: &str) -> Result<TopTracksResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.spotify.com/v1/artists/{}/top-tracks?market=US", artist_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    let top_tracks: TopTracksResponse = response.json().await?;
    Ok(top_tracks)
}

async fn top_tracks_handler(state: web::Data<Arc<AppState>>) -> impl Responder {
    let access_token = match get_access_token(&state.client_id, &state.client_secret).await {
        Ok(token) => token,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to get access token: {}", e)),
    };

    // Example artist ID for Radiohead
    let artist_id = "4Z8W4fKeB5YxbusRsdQVPb";

    match get_artist_top_tracks(&access_token, artist_id).await {
        Ok(top_tracks) => HttpResponse::Ok().json(top_tracks),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to get top tracks: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");

    let app_state = Arc::new(AppState { client_id, client_secret });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/top-tracks", web::get().to(top_tracks_handler))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
