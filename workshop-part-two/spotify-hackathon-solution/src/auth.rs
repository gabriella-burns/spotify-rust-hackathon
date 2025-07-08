use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;
use webbrowser;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

pub async fn get_spotify_token(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<AuthResponse, Box<dyn Error>> {
    let client = Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
    ];

    let response = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let auth_response: AuthResponse = response.json().await?;
        Ok(auth_response)
    } else {
        Err(format!("Error: {}", response.status()).into())
    }
}

pub async fn refresh_spotify_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<AuthResponse, Box<dyn Error>> {
    let client = Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let response = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let mut auth_response: AuthResponse = response.json().await?;
        // If the refresh token is not returned, use the old one
        if auth_response.refresh_token.is_empty() {
            auth_response.refresh_token = refresh_token.to_string();
        }
        Ok(auth_response)
    } else {
        Err(format!("Error: {}", response.status()).into())
    }
}

pub fn get_auth_code(
    client_id: &str,
    redirect_uri: &str,
    scope: &str,
) -> Result<String, Box<dyn Error>> {
    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        client_id,
        urlencoding::encode(redirect_uri),
        urlencoding::encode(scope)
    );

    // Open the authorization URL in the default web browser
    webbrowser::open(&auth_url)?;

    // Start a local server to listen for the callback
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    println!("Listening for callback on http://localhost:3000/callback");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line)?;

                // Extract the authorization code from the request
                if let Some(code) = extract_code(&request_line) {
                    // Send a response to the browser
                    let response = "HTTP/1.1 200 OK\r\n\r\nAuthorization successful! You can close this window.";
                    stream.write_all(response.as_bytes())?;

                    return Ok(code);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Err("Failed to get authorization code".into())
}

fn extract_code(request_line: &str) -> Option<String> {
    let url = request_line.split_whitespace().nth(1)?;
    let url = format!("http://localhost{}", url);
    let parsed_url = Url::parse(&url).ok()?;
    parsed_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
}
