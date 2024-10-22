mod auth;

use auth::get_auth_code;
use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::env;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Track {
    name: String,
    artists: Vec<Artist>,
}

#[derive(Deserialize, Debug)]
struct Artist {
    name: String,
}

#[derive(Deserialize, Debug)]
struct TopTracksResponse {
    items: Vec<Track>,
}

async fn get_top_tracks(access_token: &str) -> Result<TopTracksResponse, Box<dyn Error>> {
    let client: reqwest::Client = reqwest::Client::new();
    // let track_or_artist : Result<String, AbortReason> = Input::new("Enter your name", name_validation)
    // .default_value("track")
    // .help_message("Please provide your real name")
    // .display();
    let url = "https://api.spotify.com/v1/me/top/tracks";


    let response: reqwest::Response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .query(&[("time_range", "long_term"), ("limit", "50")])
        .send() 
        .await?;
    println!("top tracks: {:?}", response);

    let top_tracks: TopTracksResponse = response.json().await?;

    Ok(top_tracks)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client_id: String = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret: String =
        env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
    let redirect_uri: &str = "http://localhost:8080/callback";
    let scope: &str = "user-top-read";

    println!("Getting authorization code...");
    let auth_code: String = get_auth_code(&client_id, redirect_uri, scope)?;
    println!("Authorization code obtained successfully!");

    // Use the auth_code to get an access token
    let auth_response: auth::AuthResponse =
        auth::get_spotify_token(&client_id, &client_secret, redirect_uri, &auth_code).await?;
    let access_token: &String = &auth_response.access_token;
    println!("Access token obtained successfully!");

    println!("Fetching top tracks...");
    let top_tracks: TopTracksResponse = get_top_tracks(&access_token).await?;

    println!("Your top tracks:");
    for (i, track) in top_tracks.items.iter().enumerate() {
        let artists = track
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}. {} by {}", i + 1, track.name, artists);
    }

    Ok(())
}
