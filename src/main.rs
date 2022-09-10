use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use xshell::{cmd, Shell};
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use std::net::SocketAddr;

async fn root() -> &'static str {
    "Hello, world!"
}

async fn disable(
    Json(input): Json<DisablePihole>,
) -> Result<impl IntoResponse, StatusCode> {
    if disable_pihole(input.duration).is_ok() {
        Ok(Json(PiholeResponse{pihole_status: PiholeStatus::Disabled}))
    } else {
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
}

async fn enable() -> Result<impl IntoResponse, StatusCode> {
    if enable_pihole().is_ok() {
        Ok(Json(PiholeResponse{pihole_status: PiholeStatus::Enabled}))
    } else {
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
}

async fn status() -> impl IntoResponse {
    match pihole_status() {
        PiholeStatus::Enabled => Json(PiholeResponse{pihole_status: PiholeStatus::Enabled}),
        PiholeStatus::Disabled => Json(PiholeResponse{pihole_status: PiholeStatus::Disabled})
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/disable", post(disable))
        .route("/enable", post(enable))
        .route("/status", get(status));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn disable_pihole(duration: Option<u8>) -> Result<()> {
    dbg!(&duration);
    let sh = Shell::new().unwrap();
    let duration = if let Some(time) = duration {
        format!("{time}m")
    } else {
        String::new()
    };
    let result = cmd!(sh, "pihole disable {duration}").read()?;
    let re = Regex::new(r"Pi-hole Disabled")?;
    if re.is_match(&result) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Pi-hole is already disabled"))
    }
}
fn enable_pihole() -> Result<()> {
    let sh = Shell::new().unwrap();
    let result = cmd!(sh, "pihole enable").read()?;
    let re = Regex::new(r"Pi-hole Enabled")?;
    if re.is_match(&result) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Pi-hole is already disabled"))
    }
}
fn pihole_status() -> PiholeStatus {
    let sh = Shell::new().unwrap();
    let result = cmd!(sh, "pihole status").read().unwrap();
    let re = Regex::new(r"Pi-hole blocking is enabled").unwrap();
    if re.is_match(&result) {
        PiholeStatus::Enabled
    } else {
        PiholeStatus::Disabled
    }
}

#[derive(Deserialize, Serialize)]
struct PiholeResponse {
    pihole_status: PiholeStatus
}
#[derive(Deserialize, Serialize)]
enum PiholeStatus {
    Enabled,
    Disabled,
}

#[derive(Deserialize)]
struct DisablePihole {
    duration: Option<u8>,
}
