mod game;

use std::{collections::HashMap, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query},
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    BoxError, Json, Router,
};
use game::{Champion, Game};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize)]
struct Solution {
    champions: Vec<Champion>,
    evaluation: i32,
    traits: HashMap<i8, i8>,
}

#[derive(Debug, Deserialize)]
struct SolveParams {
    high_tier: Option<bool>,
    augment: Option<String>,
    tier_coefficient: Option<f64>,
}

fn create_game(
    champions_json: &str,
    traits_json: &str,
    high_tier: bool,
    augment: &String,
    tier_coefficient: f64,
) -> Game {
    let mut game = Game::new();
    match game.read_json(
        champions_json,
        traits_json,
        high_tier,
        &augment,
        tier_coefficient,
    ) {
        Ok(()) => {}
        Err(err) => panic!("Erro ao ler as informações {:?}", err),
    }
    game
}

async fn traits() -> impl IntoResponse {
    let game = create_game(
        "champions.json",
        "traits.json",
        true,
        &"standUnited".to_owned(),
        1.0,
    );
    Json(game.traits)
}

async fn solve(Path(num_champs): Path<u8>, Query(params): Query<SolveParams>) -> impl IntoResponse {
    let high_tier = params.high_tier.unwrap_or(false);
    let augment = params.augment.unwrap_or("standUnited".to_owned());
    let tier_coefficient = params.tier_coefficient.unwrap_or(1.0);
    let game = create_game(
        if augment == "standUnited" {
            "champions.json"
        } else {
            "champions_bd.json"
        },
        if augment == "standUnited" {
            "traits.json"
        } else {
            "traits_bd.json"
        },
        high_tier,
        &augment,
        tier_coefficient,
    );
    let mut rng = rand::thread_rng();

    let solution = vec![rng.gen_range(1..=game.pool.len()) as i8; num_champs.clamp(1, 11) as usize];
    let ils_solution = if augment == "standUnited" {
        game.ils(&solution)
    } else {
        game.ils_bd(&solution)
    };

    let mut champions = vec![];
    ils_solution
        .iter()
        .for_each(|champ_id| champions.push(game.pool[&champ_id].clone()));

    Json(Solution {
        champions,
        evaluation: if augment == "standUnited" {
            game.evaluate(&ils_solution).0
        } else {
            game.evaluate_bd(&ils_solution).0
        },
        traits: game.get_traits(&ils_solution),
    })
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let cors_layer = CorsLayer::new()
        .allow_origin(
            "https://tft-su-bd2.vercel.app"
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET]);

    let app = Router::new()
        .route("/solve/:n_champions", get(solve))
        .route("/traits", get(traits))
        .layer(cors_layer)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(10, Duration::from_secs(5))),
        );

    Ok(app.into())
}
