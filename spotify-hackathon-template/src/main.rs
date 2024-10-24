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

//types for spotify response
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
            let artists = track
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(f, "{}. {} by {}", i + 1, track.name, artists)?;
        }
        Ok(())
    }
}

// open api types

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

// get top tracks

async fn get_top_tracks(access_token: &str) -> Result<TopTracksResponse, Box<dyn Error>> {
    let client: reqwest::Client = reqwest::Client::new();
    let url = "https://api.spotify.com/v1/me/top/tracks";

    let response: reqwest::Response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .query(&[("time_range", "long_term"), ("limit", "30")])
        .send()
        .await?;
    println!("top tracks: {:?}", response);

    let top_tracks: TopTracksResponse = response.json().await?;

    Ok(top_tracks)
}

// write a new function that retrieves your top artists from the spotify api

async fn roast_or_toast_music_taste(top_tracks: &TopTracksResponse, roast: bool, celebrity: &str) {
    dotenv().unwrap();
    set_key(env::var("OPENAI_API_KEY").unwrap());

    let mut top_tracks_list = String::new();

    for (i, track) in top_tracks.items.iter().enumerate() {
        let artists = track
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let formatted_track = format!("{}. {} by {}", i + 1, track.name, artists);
        top_tracks_list.push_str(&formatted_track);
        top_tracks_list.push('\n');
    }

    println!("Your top tracks:\n{}", top_tracks_list);

    let mut prompt = format!(
        "Please {} my music taste in the style of {}. Here are my top tracks:\n{}",
        if roast { "roast" } else { "toast" },
        celebrity,
        top_tracks_list
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n")
    );

    stdin().read_line(&mut prompt).unwrap();

    let completion = Completion::builder("gpt-3.5-turbo-instruct-0914")
        .prompt(&prompt)
        // .max_tokens(90000)
        .suffix("20000")
        .create()
        .await
        .unwrap();

    let response = &completion.choices.first().unwrap().text;

    println!("\nResponse:{response}\n");
}

// create a server and define an endpoint to display the spotify data and the roast or toast response or an error message
// You can use the modules from previous workshops as a reference

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let client_id: String = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret: String =
        env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
    let redirect_uri: &str = "http://localhost:3000/callback";
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

    // Ask user if they want a roast or toast
    let roast_or_toast_choice: String = Input::new()
        .with_prompt("Do you want a roast or a toast? (roast/toast)")
        .default("roast".to_string())
        .interact_text()
        .unwrap();

    let roast = roast_or_toast_choice.to_lowercase() == "roast";

    // Ask user for a celebrity name
    let celebrity: String = Input::new()
        .with_prompt("Enter the name of a celebrity whose style you want")
        .default("Gordon Ramsay".to_string())
        .interact_text()
        .unwrap();

    // Call the function to roast or toast in the celebrity's style
    roast_or_toast_music_taste(&top_tracks, roast, &celebrity).await;

    Ok(())
}
