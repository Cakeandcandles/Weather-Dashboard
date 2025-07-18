use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{env, net::SocketAddr};
use tower_http::services::ServeDir;

#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .route("/api/weather", get(get_weather))
        .nest_service("/", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn get_weather(Query(params): Query<WeatherRequest>) -> impl IntoResponse {
    let api_key = match env::var("WEATHER_API_KEY") {
        Ok(key) => key,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "API key not set").into_response(),
    };

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        params.city, api_key
    );

    let client = Client::new();

    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_GATEWAY, "Failed to connect weather service").into_response(),
    };

    let status = res.status();
    let body = match res.text().await {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read weather data").into_response(),
    };

    (status, Json(body)).into_response()

}