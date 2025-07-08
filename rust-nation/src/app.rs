use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpotifyTrack {
    name: String,
    popularity: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthResponse {
    access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopTracksResponse {
    tracks: Vec<SpotifyTrack>,
}

#[cfg(feature = "ssr")]
#[server]
async fn get_access_token(
    client_id: String,
    client_secret: String,
) -> Result<AuthResponse, ServerFnError> {
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

    let auth_response: String = response.json().await?;
    Ok(auth_response.access_token)
}

/// Function to get the top tracks of an artist
/// This hits the API of spotify, with the artist ID and
/// the access token, and returns the top tracks of the artist
///
/// # Arguments
/// * `access_token` - A string that holds the access token
/// * `artist_id` - A string that holds the artist id
#[cfg(feature = "ssr")]
#[server]
async fn get_artist_top_tracks(
    access_token: String,
    artist_id: String,
) -> Result<TopTracksResponse, ServerFnError> {
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

/// Function that gets top artist tracks
///
/// *****************************************************
/// #TODO: ADD this to an API endpoint
/// *****************************************************
#[server]
async fn top_tracks_handler() -> Result<SpotifyTrack, ServerFnError> {
    // Load the environment variables from the .env file
    let client_id: String = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret: String =
        env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");

    // fetch the access token from spotify's api
    let access_token = match get_access_token(client_id, client_secret).await {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to get access token: {}", e))
        }
    };

    // Example artist ID for Radiohead
    let artist_id = "4Z8W4fKeB5YxbusRsdQVPb";

    // match a response or an error
    match get_artist_top_tracks(access_token, artist_id).await {
        Ok(top_tracks) => HttpResponse::Ok().json(top_tracks),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to get top tracks: {}", e))
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/rust-nation.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=move || "Not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                    <Route path=StaticSegment("top-tracks") view=TopTracksUI/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn TopTracksUI() -> impl IntoView {
    view! {
        <div>
            <h1>"Top Tracks"</h1>
            <ul>
                // {tracks.into_iter().map(|track| view! {
                //     <li>{track.name.clone()} - Popularity: {track.popularity}</li>
                // }).collect_view()}
            </ul>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
