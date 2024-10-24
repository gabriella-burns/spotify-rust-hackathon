use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;


/// Struct for the application state
/// 
/// # Arguments
/// * `client_id` - A string that holds the client id
/// * `client_secret` - A string that holds the client secret
#[derive(Clone)]
struct AppState {
    client_id: String,
    client_secret: String,
}

/// Struct for the authentication response from spotify
/// 
/// # Arguments
/// * `access_token` - A string that holds the access token
#[derive(Serialize, Deserialize, Debug)]
struct AuthResponse {
    access_token: String,
}

/// Struct for the track
/// 
/// # Arguments
/// * `name` - A string that holds the name of the track
/// * `popularity` - An unsigned 32-bit integer that holds the popularity of the track
#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    popularity: u32,
}

/// Struct for the top tracks response
/// 
/// # Arguments
/// * `tracks` - A vector of tracks
#[derive(Serialize, Deserialize, Debug)]
struct TopTracksResponse {
    tracks: Vec<Track>,
}


/// Function to greet a user. this is an API endpoint
/// with GET request method as the decorator
/// 
/// # Arguments
/// * `name` - A string that holds the name of the user (from url)
#[get("/greet/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", name)
}

/// Function to get the access token
/// 
/// # Arguments
/// * `client_id` - A string that holds the client id
/// * `client_secret` - A string that holds the client secret
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

/// Function to get the top tracks of an artist
/// This hits the API of spotify, with the artist ID and 
/// the access token, and returns the top tracks of the artist
/// 
/// # Arguments
/// * `access_token` - A string that holds the access token
/// * `artist_id` - A string that holds the artist id
/// 
/// *****************************************************
/// #TODO: ADD this to an API endpoint
/// *****************************************************
async fn get_artist_top_tracks(
    access_token: &str,
    artist_id: &str,
) -> Result<TopTracksResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.spotify.com/v1/artists/{}/top-tracks?market=US",
        artist_id
    );

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
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get access token: {}", e))
        }
    };

    // Example artist ID for Radiohead
    let artist_id = "4Z8W4fKeB5YxbusRsdQVPb";

    match get_artist_top_tracks(&access_token, artist_id).await {
        Ok(top_tracks) => HttpResponse::Ok().json(top_tracks),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to get top tracks: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Load the environment variables from the .env file
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret =
        env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");

    // Create the application state
    let app_state = Arc::new(AppState {
        client_id,
        client_secret,
    });

    // Start the server
    HttpServer::new(|| App::new()
        .service(greet)) // Add the greet API endpoint (i.e. fn greet())
        // Add the top_tracks_handler API endpoint (i.e. fn top_tracks_handler()) here!!
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
