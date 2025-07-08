mod auth;

use auth::get_auth_code;
use dialoguer::Input;
use dotenv::dotenv;
use openai::{completions::Completion, set_key};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::io::stdin;

// Types for Spotify response
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

impl std::fmt::Display for TopTracksResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, track) in self.items.iter().enumerate() {
            let artists = format_artists(&track.artists);
            writeln!(f, "{}. {} by {}", i + 1, track.name, artists)?;
        }
        Ok(())
    }
}

// OpenAI types
#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

// Configuration struct for better organization
#[derive(Clone)]
struct SpotifyConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scope: String,
}

impl SpotifyConfig {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(SpotifyConfig {
            client_id: env::var("SPOTIFY_CLIENT_ID")?,
            client_secret: env::var("SPOTIFY_CLIENT_SECRET")?,
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "user-top-read".to_string(),
        })
    }
}

// Utility functions
fn format_artists(artists: &[Artist]) -> String {
    artists
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_track_with_index(track: &Track, index: usize) -> String {
    let artists = format_artists(&track.artists);
    format!("{}. {} by {}", index + 1, track.name, artists)
}

fn format_tracks_list(tracks: &[Track], limit: Option<usize>) -> String {
    let limit = limit.unwrap_or(tracks.len());
    tracks
        .iter()
        .take(limit)
        .enumerate()
        .map(|(i, track)| format_track_with_index(track, i))
        .collect::<Vec<_>>()
        .join("\n")
}

// Spotify API functions
async fn get_top_tracks(
    access_token: &str,
    time_range: &str,
    limit: u32,
) -> Result<TopTracksResponse, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://api.spotify.com/v1/me/top/tracks";

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .query(&[("time_range", time_range), ("limit", &limit.to_string())])
        .send()
        .await?;

    println!("Top tracks response status: {}", response.status());

    let top_tracks: TopTracksResponse = response.json().await?;
    Ok(top_tracks)
}

async fn authenticate_spotify(config: &SpotifyConfig) -> Result<String, Box<dyn Error>> {
    println!("Getting authorization code...");
    let auth_code = get_auth_code(&config.client_id, &config.redirect_uri, &config.scope)?;
    println!("Authorization code obtained successfully!");

    let auth_response = auth::get_spotify_token(
        &config.client_id,
        &config.client_secret,
        &config.redirect_uri,
        &auth_code,
    )
    .await?;

    println!("Access token obtained successfully!");
    Ok(auth_response.access_token)
}

// OpenAI functions
fn initialize_openai() -> Result<(), Box<dyn Error>> {
    dotenv()?;
    let api_key = env::var("OPENAI_API_KEY")?;
    set_key(api_key);
    Ok(())
}

async fn generate_ai_response(prompt: &str, model: &str) -> Result<String, Box<dyn Error>> {
    let completion = Completion::builder(model)
        .prompt(prompt)
        // .suffix("70000")
        .create()
        .await?;

    let response = completion
        .choices
        .first()
        .ok_or("No response from OpenAI")?
        .text
        .clone();

    Ok(response)
}

// Music analysis functions
async fn roast_or_toast_music_taste(
    top_tracks: &TopTracksResponse,
    roast: bool,
    celebrity: &str,
    track_limit: usize,
) -> Result<String, Box<dyn Error>> {
    initialize_openai()?;

    let tracks_list = format_tracks_list(&top_tracks.items, Some(track_limit));

    let action = if roast { "roast" } else { "toast" };
    let prompt = format!(
        "Please write a one sentence {} of my music taste in the style of {}. Reference the track, genre, and/or artist in the list as part of the sentence. The sentence must be complete and under 50 characters. Do not use hashtags. Here are my top tracks:\n{}",
        action,
        celebrity,
        tracks_list
    );

    let response = generate_ai_response(&prompt, "gpt-3.5-turbo-instruct-0914").await?;
    println!("{}", response);

    Ok(response)
}

// User interaction functions
fn get_user_preferences() -> Result<(bool, String), Box<dyn Error>> {
    let roast_or_toast_choice: String = Input::new()
        .with_prompt("Do you want a roast or a toast? (roast/toast)")
        .default("roast".to_string())
        .interact_text()?;

    let roast = roast_or_toast_choice.to_lowercase() == "roast";

    let celebrity: String = Input::new()
        .with_prompt("Enter the name of a celebrity whose style you want")
        .default("Gordon Ramsay".to_string())
        .interact_text()?;

    Ok((roast, celebrity))
}

// Main application logic
async fn run_music_analysis() -> Result<(), Box<dyn Error>> {
    let config = SpotifyConfig::from_env()?;

    let access_token = authenticate_spotify(&config).await?;

    println!("Fetching top tracks...");
    let top_tracks = get_top_tracks(&access_token, "medium_term", 30).await?;

    println!("Your top tracks:");
    println!("{}", top_tracks);

    let (roast, celebrity) = get_user_preferences()?;

    roast_or_toast_music_taste(&top_tracks, roast, &celebrity, 5).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    run_music_analysis().await
}
