use actix_files::Files;
use actix_web::*;
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};

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

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/top-tracks" view=TopTracks/>
            </Routes>
        </Router>
    }
}

#[component]
fn TopTracks() -> impl IntoView {
    // Your existing TopTracks component logic here
    view! {
        <div>
            <h1>"Top Tracks"</h1>
                <For
                each=move || data.get()
                key=|state| state.key.clone()
                let:child
            >
                <p>{child.value}</p>
            </For>
        </div>
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(App);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                App
            )
            .service(Files::new("/", site_root))
            .wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}
