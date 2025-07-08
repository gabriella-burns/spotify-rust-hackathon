//! # Spotify Music Analysis with Rust Concepts
//!
//! This module demonstrates various Rust concepts including:
//! - **Lifetimes**: Explicit lifetime annotations for borrowed data
//! - **Ownership**: Understanding who owns data and when it's borrowed
//! - **Traits**: Defining shared behavior across different types
//! - **Error Handling**: Custom error types with proper error propagation
//! - **Iterators**: Custom iterator implementations
//! - **Generic Programming**: Type-safe abstractions

mod auth;

use auth::get_auth_code;
use dialoguer::Input;
use dotenv::dotenv;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fmt;

/// # Data Structures for Spotify API Responses
///
/// These structs represent the JSON responses from Spotify's Web API.
/// They use `#[derive(Deserialize)]` to automatically parse JSON into Rust structs.

/// Represents a single track from Spotify
///
/// **Rust Concept: Derive Macros**
/// - `#[derive(Deserialize, Debug, Clone)]` automatically implements:
///   - `Deserialize`: Converts JSON to this struct
///   - `Debug`: Allows printing with `{:?}`
///   - `Clone`: Allows creating copies of the struct
#[derive(Deserialize, Debug, Clone)]
struct Track {
    name: String,
    artists: Vec<Artist>,
}

/// Represents an artist from Spotify
///
/// **Rust Concept: Owned vs Borrowed Data**
/// - `String` is an owned type (we own the memory)
/// - `&str` would be a borrowed reference (we don't own it)
#[derive(Deserialize, Debug, Clone)]
struct Artist {
    name: String,
}

/// Represents the response from Spotify's top tracks endpoint
///
/// **Rust Concept: Collections**
/// - `Vec<Track>` is a growable array of owned `Track` instances
/// - Each `Track` in the vector is owned by the vector
#[derive(Deserialize, Debug)]
struct TopTracksResponse {
    items: Vec<Track>,
}

/// # Custom Error Type - Rust Error Handling Pattern
///
/// **Rust Concept: Custom Error Types**
/// Instead of using generic `Box<dyn Error>`, we create specific error variants
/// that provide better error messages and allow callers to handle different
/// error cases appropriately.
///
/// **Benefits:**
/// - Type-safe error handling
/// - Specific error messages for each failure mode
/// - Allows pattern matching on error types
/// - Better debugging and user experience
#[derive(Debug)]
enum MusicAnalysisError {
    /// Authentication errors from Spotify API
    SpotifyAuth(String),
    /// Errors from OpenAI API calls
    OpenAIError(String),
    /// User input validation errors
    UserInput(String),
    /// Network or HTTP errors
    NetworkError(String),
}

/// **Rust Concept: Implementing Display Trait**
/// This allows our error type to be printed with `{}` format specifier
/// and provides human-readable error messages.
impl fmt::Display for MusicAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicAnalysisError::SpotifyAuth(msg) => {
                write!(f, "Spotify authentication error: {}", msg)
            }
            MusicAnalysisError::OpenAIError(msg) => write!(f, "OpenAI API error: {}", msg),
            MusicAnalysisError::UserInput(msg) => write!(f, "User input error: {}", msg),
            MusicAnalysisError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

/// **Rust Concept: Implementing Error Trait**
/// This makes our custom error type compatible with Rust's error handling ecosystem.
/// The `Error` trait is required for `?` operator and error propagation.
impl Error for MusicAnalysisError {}

/// **Rust Concept: From Trait Implementation**
/// This allows automatic conversion from `Box<dyn Error>` to our custom error type.
/// The `From` trait is used by the `?` operator for automatic error conversion.
impl From<Box<dyn Error>> for MusicAnalysisError {
    fn from(err: Box<dyn Error>) -> Self {
        MusicAnalysisError::NetworkError(err.to_string())
    }
}

/// **Rust Concept: Implementing Display for Custom Types**
/// This allows `TopTracksResponse` to be printed directly with `println!("{}", response)`.
/// The `Formatter<'_>` uses an anonymous lifetime `'_` which means "any lifetime".
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

/// # Configuration Management
///
/// **Rust Concept: Clone Trait**
/// `#[derive(Clone)]` allows creating copies of the config without taking ownership.
#[derive(Clone)]
struct SpotifyConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scope: String,
}

impl SpotifyConfig {
    /// Creates a new config from environment variables
    ///
    /// **Rust Concept: Error Handling with Custom Types**
    /// Returns `Result<Self, MusicAnalysisError>` instead of generic error types
    /// for better error handling and debugging.
    fn from_env() -> Result<Self, MusicAnalysisError> {
        Ok(SpotifyConfig {
            client_id: env::var("SPOTIFY_CLIENT_ID").map_err(|_| {
                MusicAnalysisError::SpotifyAuth("SPOTIFY_CLIENT_ID not set".to_string())
            })?,
            client_secret: env::var("SPOTIFY_CLIENT_SECRET").map_err(|_| {
                MusicAnalysisError::SpotifyAuth("SPOTIFY_CLIENT_SECRET not set".to_string())
            })?,
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scope: "user-top-read".to_string(),
        })
    }

    /// teaching moment - Error #1: Ownership Issue - Dangling Reference
    /// This function tries to return a reference to a local variable, which would be a dangling reference
    fn get_config_reference(&self) -> &SpotifyConfig {
        let local_config = SpotifyConfig::from_env().unwrap();
        &local_config // ERROR: Can't return reference to local variable
    }
}

/// # Traits - Defining Shared Behavior
///
/// **Rust Concept: Traits**
/// Traits define shared behavior that can be implemented by multiple types.
/// This is Rust's way of achieving polymorphism and code reuse.
///
/// **Benefits:**
/// - Code reuse across different types
/// - Type-safe abstractions
/// - Compile-time polymorphism
/// - Interface-like behavior
trait Formattable {
    /// Formats the implementing type into a string representation
    fn format(&self) -> String;
}

/// **Rust Concept: Trait Implementation**
/// This implements the `Formattable` trait for the `Track` type.
/// Now any `Track` instance can call `.format()` method.
impl Formattable for Track {
    fn format(&self) -> String {
        let artists = self
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} by {}", self.name, artists)
    }
}

/// # Custom Iterator Implementation
///
/// **Rust Concept: Custom Iterators**
/// This demonstrates how to create your own iterator type that can be used
/// with Rust's iterator methods like `map`, `filter`, `collect`, etc.
///
/// **Benefits:**
/// - Encapsulates iteration logic
/// - Can be used with standard iterator methods
/// - Provides custom iteration behavior
/// - Memory efficient (doesn't copy data)

/// Custom iterator for tracks with explicit lifetime
///
/// **Rust Concept: Lifetime in Structs**
/// `'a` lifetime parameter ensures the iterator cannot outlive the data it references.
/// This prevents dangling references and ensures memory safety.
struct TrackIterator<'a> {
    tracks: &'a [Track], // Borrowed reference to tracks
    index: usize,        // Current position in the iterator
}

impl<'a> TrackIterator<'a> {
    /// Creates a new iterator over tracks
    ///
    /// **Rust Concept: Constructor Pattern**
    /// `new()` is a common Rust convention for creating new instances.
    fn new(tracks: &'a [Track]) -> Self {
        TrackIterator { tracks, index: 0 }
    }
}

/// **Rust Concept: Iterator Trait Implementation**
/// This implements the standard `Iterator` trait, making our custom type
/// compatible with all of Rust's iterator methods and for-loops.
impl<'a> Iterator for TrackIterator<'a> {
    /// **Rust Concept: Associated Types**
    /// `type Item = &'a Track;` defines what type this iterator yields.
    /// The lifetime `'a` ensures the yielded references are valid.
    type Item = &'a Track;

    /// **Rust Concept: Iterator Protocol**
    /// `next()` is called repeatedly to get the next item.
    /// Returns `Some(item)` if there's more data, `None` when done.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tracks.len() {
            let track = &self.tracks[self.index];
            self.index += 1;
            Some(track)
        } else {
            None
        }
    }
}

/// # API Functions with Enhanced Error Handling
///
/// **Rust Concept: Result Types**
/// All functions return `Result<T, MusicAnalysisError>` for proper error handling.
/// This allows callers to handle errors appropriately and provides better debugging.

/// Fetches top tracks from Spotify API
///
/// **Rust Concept: String Slices vs Owned Strings**
/// - `&str` parameters are borrowed string slices (efficient)
/// - `String` would be owned strings (requires allocation)
///
/// **Rust Concept: Error Mapping**
/// `.map_err()` converts errors from the underlying library to our custom error type.
async fn get_top_tracks(
    access_token: &str,
    time_range: &str,
    limit: u32,
) -> Result<TopTracksResponse, MusicAnalysisError> {
    let client = Client::new();
    let url = "https://api.spotify.com/v1/me/top/tracks";

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .query(&[("time_range", time_range), ("limit", &limit.to_string())])
        .send()
        .await
        .map_err(|e| MusicAnalysisError::NetworkError(e.to_string()))?;

    println!("Top tracks response status: {}", response.status());

    if response.status().is_success() {
        let top_tracks: TopTracksResponse = response
            .json()
            .await
            .map_err(|e| MusicAnalysisError::NetworkError(e.to_string()))?;
        Ok(top_tracks)
    } else {
        Err(MusicAnalysisError::SpotifyAuth(format!(
            "HTTP {}: {}",
            response.status(),
            response.status().as_str()
        )))
    }
}

/// Authenticates with Spotify and returns access token
///
/// **Rust Concept: Reference Parameters**
/// Takes `&SpotifyConfig` to borrow the config without taking ownership.
/// This allows the caller to reuse the config after this function call.
async fn authenticate_spotify(config: &SpotifyConfig) -> Result<String, MusicAnalysisError> {
    println!("Getting authorization code...");
    let auth_code = get_auth_code(&config.client_id, &config.redirect_uri, &config.scope)
        .map_err(|e| MusicAnalysisError::SpotifyAuth(e.to_string()))?;
    println!("Authorization code obtained successfully!");

    let auth_response = auth::get_spotify_token(
        &config.client_id,
        &config.client_secret,
        &config.redirect_uri,
        &auth_code,
    )
    .await
    .map_err(|e| MusicAnalysisError::SpotifyAuth(e.to_string()))?;

    println!("Access token obtained successfully!");
    Ok(auth_response.access_token)
}

/// # OpenAI Integration Functions

/// Initializes OpenAI API with environment variables
///
/// **Rust Concept: Error Propagation**
/// Uses `?` operator to automatically propagate errors up the call stack.
/// If any operation fails, the function returns early with the error.
fn initialize_openai() -> Result<(), MusicAnalysisError> {
    dotenv().map_err(|e| MusicAnalysisError::NetworkError(e.to_string()))?;
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| MusicAnalysisError::OpenAIError("OPENAI_API_KEY not set".to_string()))?;
    set_key(api_key);
    Ok(())
}

/// Generates AI response using OpenAI Chat API
///
/// **Rust Concept: Option Handling**
/// Uses `ok_or()` to convert `Option` to `Result` with a custom error message.
/// This provides better error context than unwrapping.
async fn generate_ai_response(prompt: &str, model: &str) -> Result<String, MusicAnalysisError> {
    let message = ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(prompt.to_string()),
        name: None,
        function_call: None,
    };

    let completion = ChatCompletion::builder(model, vec![message])
        .create()
        .await
        .map_err(|e| MusicAnalysisError::OpenAIError(e.to_string()))?;

    let response = completion
        .choices
        .first()
        .ok_or(MusicAnalysisError::OpenAIError(
            "No response from OpenAI".to_string(),
        ))?
        .message
        .content
        .as_ref()
        .ok_or(MusicAnalysisError::OpenAIError(
            "No content in response".to_string(),
        ))?
        .clone();

    Ok(response)
}

/// # Music Analysis with Custom Iterator Usage
///
/// **Rust Concept: Iterator Methods**
/// Demonstrates using our custom `TrackIterator` with standard iterator methods
/// like `take()`, `enumerate()`, `map()`, and `collect()`.

/// Analyzes music taste and generates roast/toast using AI
///
/// **Rust Concept: Iterator Chain**
/// Chains multiple iterator methods together for efficient data processing:
/// 1. `TrackIterator::new()` - Creates our custom iterator
/// 2. `take(track_limit)` - Limits the number of tracks
/// 3. `enumerate()` - Adds indices to each track
/// 4. `map()` - Transforms each track to formatted string
/// 5. `collect()` - Gathers results into a vector
/// 6. `join("\n")` - Combines into a single string
async fn roast_or_toast_music_taste(
    top_tracks: &TopTracksResponse,
    roast: bool,
    celebrity: &str,
    track_limit: usize,
) -> Result<String, MusicAnalysisError> {
    initialize_openai()?;

    // Using our custom iterator with iterator methods
    let track_iterator = TrackIterator::new(&top_tracks.items);
    let tracks_list: String = track_iterator
        .take(track_limit)
        .enumerate()
        .map(|(i, track)| format!("{}. {}", i + 1, track.format()))
        .collect::<Vec<_>>()
        .join("\n");

    let action = if roast { "roast" } else { "toast" };
    let prompt = format!(
        "Please write a one sentence {} of my music taste in the style of {}. Reference the track, genre, and/or artist in the list as part of the sentence. The sentence must be complete and under 50 characters. Do not use hashtags. Here are my top tracks:\n{}",
        action,
        celebrity,
        tracks_list
    );

    let response = generate_ai_response(&prompt, "gpt-3.5-turbo").await?;
    println!("{}", response);

    Ok(response)
}

/// # User Interaction Functions
///
/// **Rust Concept: Error Handling with User Input**
/// Demonstrates proper error handling for user input operations.

/// Gets user preferences for roast/toast and celebrity style
///
/// **Rust Concept: Tuple Returns**
/// Returns `(bool, String)` tuple for multiple values without creating a struct.
/// This is a common Rust pattern for simple multi-value returns.
fn get_user_preferences() -> Result<(bool, String), MusicAnalysisError> {
    let roast_or_toast_choice: String = Input::new()
        .with_prompt("Do you want a roast or a toast? (roast/toast)")
        .default("roast".to_string())
        .interact_text()
        .map_err(|e| MusicAnalysisError::UserInput(e.to_string()))?;

    let roast = roast_or_toast_choice.to_lowercase() == "roast";

    let celebrity: String = Input::new()
        .with_prompt("Enter the name of a celebrity whose style you want")
        .default("Gordon Ramsay".to_string())
        .interact_text()
        .map_err(|e| MusicAnalysisError::UserInput(e.to_string()))?;

    Ok((roast, celebrity))
}

/// # Main Application Logic
///
/// **Rust Concept: Async/Await**
/// Demonstrates asynchronous programming with proper error handling.

/// Main application logic with ownership patterns
///
/// **Rust Concept: Ownership Flow**
/// 1. `SpotifyConfig::from_env()` - Creates owned config
/// 2. `authenticate_spotify(&config)` - Borrows config
/// 3. `get_top_tracks()` - Uses borrowed access token
/// 4. `roast_or_toast_music_taste()` - Borrows tracks data
///
/// **Rust Concept: Error Propagation**
/// Uses `?` operator throughout to propagate errors up to main function.
async fn run_music_analysis() -> Result<(), MusicAnalysisError> {
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

/// # Application Entry Point
///
/// **Rust Concept: Error Type Conversion**
/// Converts our custom error type to the generic error type expected by main.
/// This allows our custom error handling while maintaining compatibility.

/// Main function with proper error handling
///
/// **Rust Concept: Error Type Conversion**
/// `map_err(|e| Box::new(e) as Box<dyn std::error::Error>)` converts our custom
/// error type to the generic error type that main expects.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    run_music_analysis()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

// ============================================================================
// INTENTIONAL ERRORS FOR TEACHING EXERCISES
// ============================================================================

/// ERROR #1: Ownership Issue - Moving a borrowed value
///
/// **Teaching Point**: This demonstrates the classic ownership error where we try to move
/// a value that we only borrowed. The function takes ownership of `config` but then
/// tries to return a reference to it, which would be a dangling reference.
///
/// **Expected Error**: "cannot return reference to local variable `config`"
/// **Rust Concept**: Ownership and borrowing rules
fn ownership_error_example() -> &SpotifyConfig {
    let config = SpotifyConfig::from_env().unwrap();
    &config // ERROR: Can't return reference to local variable
}

/// ERROR #2: Lifetime Issue - Missing lifetime annotation
///
/// **Teaching Point**: This function returns a reference but doesn't specify how long
/// that reference is valid. Rust needs to know the lifetime to ensure memory safety.
///
/// **Expected Error**: "missing lifetime specifier"
/// **Rust Concept**: Lifetimes and reference validity
fn lifetime_error_example(tracks: &[Track]) -> &Track {
    &tracks[0] // ERROR: Missing lifetime annotation
}

/// ERROR #3: Borrowing Violation - Multiple mutable borrows
///
/// **Teaching Point**: This demonstrates the borrowing rules violation where we try to
/// have multiple mutable references to the same data simultaneously.
///
/// **Expected Error**: "cannot borrow `*tracks` as mutable more than once at a time"
/// **Rust Concept**: Borrowing rules and mutability
fn borrowing_error_example(tracks: &mut Vec<Track>) {
    let first = &mut tracks[0]; // First mutable borrow
    let second = &mut tracks[1]; // ERROR: Second mutable borrow while first is still active
    println!("{} and {}", first.name, second.name);
}

/// ERROR #4: Trait Implementation Error - Wrong type
///
/// **Teaching Point**: This tries to implement a trait for a type that doesn't match
/// the trait's requirements. The trait expects `&self` but we're trying to implement
/// it for a type that takes ownership.
///
/// **Expected Error**: "method `format` has a `&self` receiver but the function signature"
/// **Rust Concept**: Trait implementations and method signatures
impl Formattable for String {
    fn format(self) -> String {
        // ERROR: Should be `&self`, not `self`
        self
    }
}

/// ERROR #5: Iterator Misuse - Using moved value
///
/// **Teaching Point**: This demonstrates trying to use a value after it has been moved
/// into an iterator. Once a value is moved, it can't be used again.
///
/// **Expected Error**: "use of moved value"
/// **Rust Concept**: Iterator ownership and move semantics
fn iterator_error_example() {
    let tracks = vec![Track {
        name: "Song 1".to_string(),
        artists: vec![Artist {
            name: "Artist 1".to_string(),
        }],
    }];

    let iterator = tracks.into_iter(); // Moves ownership of tracks
    println!("First track: {:?}", tracks); // ERROR: tracks was moved
}

/// # Music Analysis Service
///
/// **Rust Concept: Struct with Generic Type Parameter**
/// `MusicAnalyzer<T>` can work with any type `T` that implements the required traits.
/// This provides flexibility and code reuse.
pub struct MusicAnalyzer<T> {
    client: T,
    openai_client: openai::Client,
}

impl<T> MusicAnalyzer<T> {
    /// Creates a new music analyzer with the given client
    ///
    /// **Rust Concept: Generic Type Constraints**
    /// The `where` clause ensures `T` has the required methods for music analysis.
    pub fn new(client: T, openai_api_key: String) -> Self
    where
        T: SpotifyClientTrait,
    {
        let openai_client = openai::Client::new(openai_api_key);
        MusicAnalyzer {
            client,
            openai_client,
        }
    }

    /// teaching moment - Error #4: Trait Implementation Mistake
    /// This tries to implement a trait method that doesn't match the trait definition
    fn analyze_with_wrong_signature(&self, tracks: Vec<Track>) -> String {
        // ERROR: This method signature doesn't match the trait definition
        // The trait expects &self but this tries to take ownership of tracks
        "analysis".to_string()
    }

    /// Analyzes top tracks using OpenAI's chat completion API
    ///
    /// **Rust Concept: Iterator Methods**
    /// Uses `map()`, `collect()`, and `join()` for functional programming style data transformation.
    pub async fn analyze_top_tracks(&self, tracks: &[Track]) -> Result<String, MusicAnalysisError> {
        let track_info: Vec<String> = tracks
            .iter()
            .map(|track| {
                let artists = track
                    .artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} by {}", track.name, artists)
            })
            .collect();

        let prompt = format!(
            "Analyze these top tracks and provide insights about the user's music taste: {}",
            track_info.join(", ")
        );

        let messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: prompt,
        }];

        let response = self
            .openai_client
            .chat()
            .create(ChatCompletionRequest {
                model: "gpt-3.5-turbo".to_string(),
                messages,
                ..Default::default()
            })
            .await
            .map_err(|e| MusicAnalysisError::OpenAIError(e.to_string()))?;

        Ok(response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "No analysis available".to_string()))
    }
}

impl SpotifyClient {
    /// Creates a new Spotify client with the given configuration
    ///
    /// **Rust Concept: Lifetime Annotations**
    /// The `'a` lifetime ensures the client cannot outlive the config it references.
    /// This prevents dangling references and memory safety issues.
    pub fn new(config: &'a SpotifyConfig) -> Self {
        SpotifyClient { config }
    }

    /// teaching moment - Error #2: Lifetime Annotation Issue
    /// This function has incorrect lifetime annotations that don't match the actual lifetimes
    fn analyze_tracks_with_wrong_lifetime<'b>(&'a self, tracks: &'b Vec<Track>) -> &'b str {
        // ERROR: Lifetime 'b doesn't match the actual lifetime of the data
        "analysis result"
    }

    /// Generates the authorization URL for Spotify OAuth
    ///
    /// **Rust Concept: String Formatting**
    /// Uses `format!` macro for efficient string concatenation instead of multiple `+` operations.
    pub fn get_auth_url(&self) -> String {
        format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
            self.config.client_id,
            self.config.redirect_uri,
            self.config.scope
        )
    }

    /// Exchanges authorization code for access token
    ///
    /// **Rust Concept: Error Handling with Custom Types**
    /// Uses `MusicAnalysisError` for specific error handling instead of generic errors.
    pub async fn get_access_token(&self, code: &str) -> Result<String, MusicAnalysisError> {
        let client = reqwest::Client::new();

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| MusicAnalysisError::SpotifyAuth(e.to_string()))?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| MusicAnalysisError::SpotifyAuth(e.to_string()))?;

        // teaching moment - Error #3: Borrowing Violation
        // This tries to borrow the token_response mutably while also borrowing it immutably
        let token_ref = &token_response.access_token;
        let mut token_copy = token_response; // ERROR: Can't move token_response while token_ref exists
        token_copy.access_token = "modified".to_string();

        Ok(token_response.access_token)
    }
}
