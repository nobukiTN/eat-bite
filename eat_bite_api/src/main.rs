use axum::{extract::State, routing::post, Json, Router, http::{Method, StatusCode}};
use tower_http::cors::{CorsLayer, Any};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::TcpListener, sync::oneshot};

// ============================
// 型定義
// ============================

#[derive(Debug, Deserialize)]
struct GuessRequest {
    guess: String,
}

#[derive(Debug, Serialize)]
struct GuessResponse {
    player_result: String,
    bot_guess: String,
    bot_result: String,
}

#[derive(Debug, Deserialize)]
struct InitRequest {
    player_secret: String,
}

#[derive(Debug)]
struct GameBot {
    memory: Vec<(Vec<u8>, (u8, u8))>,
}

impl GameBot {
    fn new() -> Self {
        GameBot { memory: Vec::new() }
    }

    fn remember(&mut self, guess: &Vec<u8>, result: (u8, u8)) {
        self.memory.push((guess.clone(), result));
    }

    fn filter_candidates(&self) -> Vec<Vec<u8>> {
        let mut all = vec![];
        for a in 0..10 {
            for b in 0..10 {
                if b == a { continue; }
                for c in 0..10 {
                    if c == a || c == b { continue; }
                    all.push(vec![a, b, c]);
                }
            }
        }

        all.into_iter()
            .filter(|candidate| {
                self.memory.iter().all(|(guess, result)| {
                    judge(candidate, guess) == *result
                })
            })
            .collect()
    }

    fn generate_guess(&self) -> Vec<u8> {
        let candidates = self.filter_candidates();
        if candidates.is_empty() {
            vec![9, 9, 9] // fallback guess (無効な guess)
        } else {
            let mut rng = thread_rng();
            candidates.choose(&mut rng).unwrap().clone()
        }
    }
}

fn parse_guess(input: &str) -> Vec<u8> {
    input.chars().filter_map(|c| c.to_digit(10)).map(|d| d as u8).collect()
}

fn judge(answer: &Vec<u8>, guess: &Vec<u8>) -> (u8, u8) {
    let mut eat = 0;
    let mut bite = 0;

    for i in 0..3 {
        if answer[i] == guess[i] {
            eat += 1;
        } else if answer.contains(&guess[i]) {
            bite += 1;
        }
    }

    (eat, bite)
}

fn generate_answer() -> Vec<u8> {
    let mut digits: Vec<u8> = (0..=9).collect();
    let mut rng = thread_rng();
    digits.shuffle(&mut rng);
    digits[..3].to_vec()
}

// ============================
// グローバル状態（AppState）
// ============================

#[derive(Clone)]
struct AppState {
    bot_secret: Vec<u8>,
    player_secret: Arc<Mutex<Vec<u8>>>,
    bot: Arc<Mutex<GameBot>>,
    turn: Arc<Mutex<u32>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

// ============================
// ハンドラー
// ============================

async fn init_handler(
    State(state): State<AppState>,
    Json(payload): Json<InitRequest>,
) -> StatusCode {
    let secret = parse_guess(&payload.player_secret);
    if secret.len() != 3 {
        return StatusCode::BAD_REQUEST;
    }

    let mut player_secret = state.player_secret.lock().unwrap();
    *player_secret = secret;

    StatusCode::OK
}

async fn guess_handler(
    State(state): State<AppState>,
    Json(payload): Json<GuessRequest>,
) -> Json<GuessResponse> {
    let player_guess = parse_guess(&payload.guess);
    let bot_secret = &state.bot_secret;
    let player_secret = state.player_secret.lock().unwrap();
    let mut bot = state.bot.lock().unwrap();
    let mut turn = state.turn.lock().unwrap();

    let (eat_p, bite_p) = judge(bot_secret, &player_guess);

    if eat_p == 3 {
        return Json(GuessResponse {
            player_result: format!("{} Eat, {} Bite", eat_p, bite_p),
            bot_guess: "---".to_string(),
            bot_result: " あなたの勝利！".to_string(),
        });
    }

    let bot_guess = bot.generate_guess();
    let (eat_b, bite_b) = judge(&player_secret, &bot_guess);
    bot.remember(&bot_guess, (eat_b, bite_b));

    *turn += 1;

    let player_result = format!("{} Eat, {} Bite", eat_p, bite_p);
    let bot_result = if eat_b == 3 {
        "🤖 Botの勝利！".to_string()
    } else {
        format!("{} Eat, {} Bite", eat_b, bite_b)
    };

    Json(GuessResponse {
        player_result,
        bot_guess: bot_guess.iter().map(|d| d.to_string()).collect::<String>(),
        bot_result,
    })
}

async fn restart_handler(State(state): State<AppState>) -> StatusCode {
    let mut bot = state.bot.lock().unwrap();
    let mut player_secret = state.player_secret.lock().unwrap();
    let mut turn = state.turn.lock().unwrap();

    *bot = GameBot::new();
    *player_secret = vec![0, 0, 0];
    *turn = 1;

    StatusCode::OK
}

async fn shutdown_handler(State(state): State<AppState>) -> StatusCode {
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

// ============================
// 起動
// ============================

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel::<()>();

    let shared_state = AppState {
        bot_secret: generate_answer(),
        player_secret: Arc::new(Mutex::new(vec![5, 2, 7])),
        bot: Arc::new(Mutex::new(GameBot::new())),
        turn: Arc::new(Mutex::new(1)),
        shutdown_tx: Arc::new(Mutex::new(Some(tx))),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/init", post(init_handler))
        .route("/api/guess", post(guess_handler))
        .route("/api/restart", post(restart_handler))
        .route("/api/shutdown", post(shutdown_handler))
        .with_state(shared_state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("🚀 APIサーバ起動: http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            rx.await.ok();
            println!(" シャットダウン要求を受け取りました。");
        })
        .await
        .unwrap();
}
