use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::engine::{ChessEngine, Engine, SearchParams, SquareMove};
use crate::engine::piece::Color;

// ---------------------------------------------------------------------------
// Lichess event stream types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum LichessEvent {
    Challenge { challenge: ChallengeInfo },
    #[allow(dead_code)]
    ChallengeDeclined { challenge: ChallengeInfo },
    GameStart { game: GameStartInfo },
    GameFinish { game: GameStartInfo },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct ChallengeInfo {
    id: String,
    variant: Variant,
}

#[derive(Debug, Deserialize)]
struct Variant {
    key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameStartInfo {
    game_id: String,
    color: String,
}

// ---------------------------------------------------------------------------
// Game stream types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum GameEvent {
    GameFull(GameFull),
    GameState(GameState),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct GameFull {
    state: GameState,
}

#[derive(Debug, Deserialize)]
struct GameState {
    moves: String,
    wtime: u64,
    btime: u64,
    winc: u64,
    binc: u64,
    status: String,
}

// ---------------------------------------------------------------------------
// HTTP request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ChallengeReq {
    username: String,
    time: String,
    color: String,
    rated: bool,
}

#[derive(Serialize)]
struct StatusResp {
    connected: bool,
    auto_accept: bool,
    active_games: Vec<String>,
}

#[derive(Serialize)]
struct OkResp {
    ok: bool,
}

// ---------------------------------------------------------------------------
// Shared bot state
// ---------------------------------------------------------------------------

struct BotState {
    connected: AtomicBool,
    auto_accept: Mutex<bool>,
    active_games: Mutex<HashSet<String>>,
}

impl Default for BotState {
    fn default() -> Self {
        BotState {
            connected: AtomicBool::new(false),
            auto_accept: Mutex::new(false),
            active_games: Mutex::new(HashSet::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP client wrapper
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct LichessClient {
    http: reqwest::Client,
    token: String,
}

impl LichessClient {
    fn new(token: String) -> Self {
        LichessClient {
            http: reqwest::Client::new(),
            token,
        }
    }

    async fn get_stream(&self, path: &str) -> anyhow::Result<reqwest::Response> {
        loop {
            let resp = self
                .http
                .get(format!("https://lichess.org{path}"))
                .bearer_auth(&self.token)
                .send()
                .await?;
            if resp.status().as_u16() == 429 {
                eprintln!("[lichess] rate-limited on GET {path}, sleeping 60s");
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
            return Ok(resp.error_for_status()?);
        }
    }

    async fn post_empty(&self, path: &str) -> anyhow::Result<()> {
        loop {
            let resp = self
                .http
                .post(format!("https://lichess.org{path}"))
                .bearer_auth(&self.token)
                .send()
                .await?;
            if resp.status().as_u16() == 429 {
                eprintln!("[lichess] rate-limited on POST {path}, sleeping 60s");
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
            resp.error_for_status()?;
            return Ok(());
        }
    }

    async fn post_form(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> anyhow::Result<serde_json::Value> {
        loop {
            let resp = self
                .http
                .post(format!("https://lichess.org{path}"))
                .bearer_auth(&self.token)
                .form(params)
                .send()
                .await?;
            if resp.status().as_u16() == 429 {
                eprintln!("[lichess] rate-limited on POST {path}, sleeping 60s");
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
            let resp = resp.error_for_status()?;
            return Ok(resp.json().await?);
        }
    }

    async fn accept_challenge(&self, id: &str) -> anyhow::Result<()> {
        self.post_empty(&format!("/api/challenge/{id}/accept")).await
    }

    async fn send_challenge(
        &self,
        username: &str,
        limit_secs: u64,
        inc_secs: u64,
        color: &str,
        rated: bool,
    ) -> anyhow::Result<String> {
        let body = self
            .post_form(
                &format!("/api/challenge/{username}"),
                &[
                    ("rated", rated.to_string()),
                    ("clock.limit", limit_secs.to_string()),
                    ("clock.increment", inc_secs.to_string()),
                    ("color", color.to_string()),
                    ("variant", "standard".to_string()),
                ],
            )
            .await?;
        body["challenge"]["id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("unexpected challenge response: {body}"))
    }

    async fn make_move(&self, game_id: &str, mv: &str) -> anyhow::Result<()> {
        self.post_empty(&format!("/api/bot/game/{game_id}/move/{mv}"))
            .await
    }

    async fn resign_game(&self, game_id: &str) -> anyhow::Result<()> {
        self.post_empty(&format!("/api/bot/game/{game_id}/resign"))
            .await
    }
}

// ---------------------------------------------------------------------------
// NDJSON line reader
// ---------------------------------------------------------------------------

async fn next_json_line(
    buf: &mut String,
    stream: &mut (impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin),
) -> anyhow::Result<Option<String>> {
    loop {
        if let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            *buf = buf[pos + 1..].to_string();
            if !line.is_empty() {
                return Ok(Some(line));
            }
            // empty line = keep-alive, try again
            continue;
        }
        match stream.next().await {
            None => return Ok(None),
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(chunk)) => buf.push_str(&String::from_utf8_lossy(&chunk)),
        }
    }
}

// ---------------------------------------------------------------------------
// Game logic
// ---------------------------------------------------------------------------

fn is_game_over(status: &str) -> bool {
    matches!(
        status,
        "mate"
            | "resign"
            | "stalemate"
            | "timeout"
            | "draw"
            | "outoftime"
            | "cheat"
            | "noStart"
            | "unknownFinish"
            | "variantEnd"
            | "aborted"
    )
}

async fn handle_state(
    client: &LichessClient,
    game_id: &str,
    our_color: Color,
    state: &GameState,
) -> anyhow::Result<()> {
    let moves: Vec<&str> = if state.moves.is_empty() {
        vec![]
    } else {
        state.moves.split_whitespace().collect()
    };

    let our_turn = match our_color {
        Color::White => moves.len() % 2 == 0,
        Color::Black => moves.len() % 2 == 1,
    };

    if !our_turn {
        return Ok(());
    }

    let parsed_moves: Vec<SquareMove> = moves
        .iter()
        .filter_map(|s| SquareMove::from_uci(s))
        .collect();

    let wtime = state.wtime;
    let btime = state.btime;
    let winc = state.winc;
    let binc = state.binc;

    let best_move_str = tokio::task::spawn_blocking(move || -> Option<String> {
        let mut engine = Engine::new();
        engine.set_position(None, &parsed_moves);
        let params = SearchParams {
            depth: None,
            movetime: None,
            wtime: Some(wtime),
            btime: Some(btime),
            winc: Some(winc),
            binc: Some(binc),
            infinite: false,
            ponder: false,
        };
        let rx = engine.start_search(params);
        rx.recv().ok().flatten().map(|mv| mv.to_string())
    })
    .await
    .map_err(|e| anyhow::anyhow!("engine panicked: {e}"))?;

    match best_move_str {
        Some(mv) => {
            eprintln!("[game {game_id}] playing {mv}");
            client.make_move(game_id, &mv).await?;
        }
        None => {
            eprintln!("[game {game_id}] engine returned no move, resigning");
            client.resign_game(game_id).await?;
        }
    }

    Ok(())
}

async fn play_game(
    client: Arc<LichessClient>,
    state: Arc<BotState>,
    game_id: String,
    color_str: String,
) {
    let our_color = if color_str == "white" {
        Color::White
    } else {
        Color::Black
    };

    state
        .active_games
        .lock()
        .unwrap()
        .insert(game_id.clone());

    if let Err(e) = play_game_inner(&client, &game_id, our_color).await {
        eprintln!("[game {game_id}] error: {e:#}");
    }

    state.active_games.lock().unwrap().remove(&game_id);
    eprintln!("[game {game_id}] done");
}

async fn play_game_inner(
    client: &LichessClient,
    game_id: &str,
    our_color: Color,
) -> anyhow::Result<()> {
    let resp = client
        .get_stream(&format!("/api/bot/game/stream/{game_id}"))
        .await?;
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    let first_line = next_json_line(&mut buf, &mut stream)
        .await?
        .ok_or_else(|| anyhow::anyhow!("game stream closed immediately"))?;

    let first_event: GameEvent = serde_json::from_str(&first_line)?;
    let initial_state = match first_event {
        GameEvent::GameFull(full) => full.state,
        _ => anyhow::bail!("expected gameFull as first event, got: {first_line}"),
    };

    handle_state(client, game_id, our_color, &initial_state).await?;

    loop {
        let line = match next_json_line(&mut buf, &mut stream).await? {
            Some(l) => l,
            None => break,
        };

        let event: GameEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[game {game_id}] parse error ({e}): {line}");
                continue;
            }
        };

        match event {
            GameEvent::GameState(gs) => {
                if is_game_over(&gs.status) {
                    eprintln!("[game {game_id}] over: {}", gs.status);
                    break;
                }
                handle_state(client, game_id, our_color, &gs).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Lichess event loop
// ---------------------------------------------------------------------------

async fn run_event_loop(
    client: &Arc<LichessClient>,
    state: &Arc<BotState>,
) -> anyhow::Result<()> {
    state.connected.store(true, Ordering::Relaxed);
    let resp = client.get_stream("/api/stream/event").await?;
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    loop {
        let line = match next_json_line(&mut buf, &mut stream).await? {
            Some(l) => l,
            None => break,
        };

        let event: LichessEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[lichess] unrecognised event ({e}): {line}");
                continue;
            }
        };

        match event {
            LichessEvent::Challenge { challenge } => {
                let auto = *state.auto_accept.lock().unwrap();
                let busy = !state.active_games.lock().unwrap().is_empty();
                if auto && !busy && challenge.variant.key == "standard" {
                    eprintln!("[lichess] accepting challenge {}", challenge.id);
                    if let Err(e) = client.accept_challenge(&challenge.id).await {
                        eprintln!("[lichess] accept failed: {e:#}");
                    }
                } else if busy {
                    eprintln!("[lichess] ignoring challenge {} (already in a game)", challenge.id);
                }
            }
            LichessEvent::GameStart { game } => {
                let busy = !state.active_games.lock().unwrap().is_empty();
                if busy {
                    eprintln!("[lichess] game {} started but already busy, resigning", game.game_id);
                    let c = client.clone();
                    let id = game.game_id;
                    tokio::spawn(async move {
                        if let Err(e) = c.resign_game(&id).await {
                            eprintln!("[lichess] resign failed: {e:#}");
                        }
                    });
                } else {
                    eprintln!("[lichess] game started: {} as {}", game.game_id, game.color);
                    let c = client.clone();
                    let s = state.clone();
                    tokio::spawn(async move {
                        play_game(c, s, game.game_id, game.color).await;
                    });
                }
            }
            LichessEvent::GameFinish { game } => {
                state.active_games.lock().unwrap().remove(&game.game_id);
            }
            _ => {}
        }
    }

    state.connected.store(false, Ordering::Relaxed);
    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP server (axum)
// ---------------------------------------------------------------------------

type AppState = (Arc<LichessClient>, Arc<BotState>);

async fn handle_status(State((_, state)): State<AppState>) -> impl IntoResponse {
    let connected = state.connected.load(Ordering::Relaxed);
    let auto_accept = *state.auto_accept.lock().unwrap();
    let active_games = state
        .active_games
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    Json(StatusResp {
        connected,
        auto_accept,
        active_games,
    })
}

async fn handle_challenge(
    State((client, _)): State<AppState>,
    Json(req): Json<ChallengeReq>,
) -> impl IntoResponse {
    match parse_time_control(&req.time) {
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
        Ok((limit_secs, inc_secs)) => {
            match client
                .send_challenge(&req.username, limit_secs, inc_secs, &req.color, req.rated)
                .await
            {
                Ok(id) => (StatusCode::OK, Json(serde_json::json!({"challenge_id": id}))),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
    }
}

async fn handle_accept_on(State((_, state)): State<AppState>) -> impl IntoResponse {
    *state.auto_accept.lock().unwrap() = true;
    Json(OkResp { ok: true })
}

async fn handle_accept_off(State((_, state)): State<AppState>) -> impl IntoResponse {
    *state.auto_accept.lock().unwrap() = false;
    Json(OkResp { ok: true })
}

fn parse_time_control(s: &str) -> anyhow::Result<(u64, u64)> {
    let (mins, inc) = s
        .split_once('+')
        .ok_or_else(|| anyhow::anyhow!("time must be 'minutes+increment', e.g. '5+3'"))?;
    Ok((mins.trim().parse::<u64>()? * 60, inc.trim().parse()?))
}

async fn serve_http(client: Arc<LichessClient>, state: Arc<BotState>, port: u16) {
    let app = Router::new()
        .route("/status", get(handle_status))
        .route("/challenge", post(handle_challenge))
        .route("/accept", post(handle_accept_on).delete(handle_accept_off))
        .with_state((client, state));

    let listener = match tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[http] failed to bind port {port}: {e}");
            return;
        }
    };
    eprintln!("[http] listening on http://127.0.0.1:{port}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[http] server error: {e}");
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn run(token: String, port: u16) -> anyhow::Result<()> {
    let client = Arc::new(LichessClient::new(token));
    let state = Arc::new(BotState::default());

    tokio::spawn(serve_http(client.clone(), state.clone(), port));

    loop {
        match run_event_loop(&client, &state).await {
            Ok(()) => break,
            Err(e) => {
                eprintln!("[lichess] event stream error: {e:#}, reconnecting in 5s");
                state.connected.store(false, Ordering::Relaxed);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}
