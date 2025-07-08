# Workshop Part 2: API Integration & Feature Development

## Session Overview

**Duration**: 2 hours (Afternoon Session)  
**Focus**: Adding new functionality using Spotify and OpenAI APIs  
**Learning Method**: Implementing new features and API integrations

## Learning Objectives

By the end of Part 2, participants will be able to:
- Use async/await for concurrent programming
- Integrate external APIs using Rust crates
- Implement new features using real-world APIs
- Handle complex API responses and error scenarios

## Session Structure

### **Async Programming Deep Dive** (30 minutes)
- Understanding async/await in Rust
- Working with Futures and async functions
- HTTP client usage with `reqwest`

### **API Integration Patterns** (30 minutes)
- REST API integration best practices
- JSON serialization/deserialization with `serde`
- Authentication and token management

### **Feature Development** (60 minutes)
- Implementing new API integrations
- Building new features using Spotify and OpenAI APIs
- Testing and debugging API calls

## New API Integration Opportunities

Based on the Spotify Web API and OpenAI documentation, here are 4 new features participants can implement:

### **Feature 1: Music Mood Analysis with AI**
- **Spotify API**: Get user's recently played tracks
- **OpenAI API**: Analyze emotional patterns in music choices
- **Result**: AI-generated insights about user's emotional state through music

### **Feature 2: Genre-Based Playlist Generator**
- **Spotify API**: Get user's top artists and create playlists
- **OpenAI API**: Generate creative playlist descriptions
- **Result**: AI-curated playlists with personalized descriptions

### **Feature 3: Music Taste Comparison & Social Features**
- **Spotify API**: Get user's top tracks and artists
- **OpenAI API**: Generate personalized music recommendations
- **Result**: Taste compatibility scores and social insights

### **Feature 4: Advanced Music Analytics Dashboard**
- **Spotify API**: Get detailed track and artist information
- **OpenAI API**: Generate comprehensive music analysis reports
- **Result**: Detailed music taste analytics with visualizable data

## Technical Implementation Guide

### **Async Programming Patterns**
```rust
// Example: Fetching multiple API endpoints concurrently
async fn fetch_user_data(access_token: &str) -> Result<UserData, MusicAnalysisError> {
    let (tracks, artists) = tokio::join!(
        get_top_tracks(access_token),
        get_top_artists(access_token)
    );
    
    Ok(UserData { tracks: tracks?, artists: artists? })
}
```

### **Error Handling for API Calls**
```rust
// Example: Robust error handling for external APIs
async fn call_spotify_api<T>(endpoint: &str, token: &str) -> Result<T, MusicAnalysisError> {
    let response = reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    match response.status() {
        StatusCode::OK => response.json::<T>().await.map_err(|e| MusicAnalysisError::NetworkError(e.to_string())),
        _ => Err(MusicAnalysisError::NetworkError(format!("HTTP {}", response.status())))
    }
}
```

## Hands-on Development Tasks

- **Task 1**: Implement basic API integration
- **Task 2**: Add error handling & resilience
- **Task 3**: Enhance with AI features
- **Task 4**: Polish & optimization

## API Reference Links

### **Spotify Web API**
- [Get User's Top Artists and Tracks](https://developer.spotify.com/documentation/web-api/reference/get-users-top-artists-and-tracks)
- [Get Recently Played Tracks](https://developer.spotify.com/documentation/web-api/reference/get-recently-played)
- [Create Playlist](https://developer.spotify.com/documentation/web-api/reference/create-playlist)
- [Get Audio Features](https://developer.spotify.com/documentation/web-api/reference/get-audio-features)

### **OpenAI API**
- [Chat Completions](https://platform.openai.com/docs/api-reference/chat)
- [Text Completions](https://platform.openai.com/docs/api-reference/completions)
- [Rate Limits](https://platform.openai.com/docs/guides/rate-limits)

## Expected Outcomes

By the end of Part 2, participants will have:
- Implemented at least one new API integration
- Experience with async programming in Rust
- Confidence in building real-world API integrations
- A functional feature that extends the original application

## Resources

- **Rust Book Chapters**:
  - [Chapter 16: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
  - [Chapter 10: Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
  - [Chapter 9: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- reqwest Documentation: HTTP client for Rust
- serde Documentation: Serialization framework
- tokio Documentation: Async runtime for Rust
