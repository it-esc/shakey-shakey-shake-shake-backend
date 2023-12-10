use std::sync::Arc;

use axum::{
    extract::State,
    response::{sse::Event, IntoResponse},
    routing::{get, post}, TypedHeader, headers::{Authorization, authorization::Bearer}, http::StatusCode,
};
use futures::{channel::mpsc::UnboundedSender, SinkExt, StreamExt};
use tokio::sync::Mutex;

macro_rules! read_n {
    ($fn_name:ident, $field: ident) => {
        async fn $fn_name(State(app): State<Arc<App>>) -> impl IntoResponse {
            *app.$field.lock().await += 1;

            app.try_send_update().await;

            "OK"
        }
    };
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

#[derive(Default)]
struct App {
    score_1: Mutex<u32>,
    score_2: Mutex<u32>,
    score_3: Mutex<u32>,
    score_4: Mutex<u32>,
    tx: Mutex<Option<UnboundedSender<Msg>>>,
}

#[derive(serde::Serialize)]
struct Msg {
    score_1: u32,
    score_2: u32,
    score_3: u32,
    score_4: u32,
}

impl App {
    pub async fn set_writer(&self, tx: UnboundedSender<Msg>) {
        *self.tx.lock().await = Some(tx);
    }

    async fn get_message(&self) -> Msg {
        Msg {
            score_1: *self.score_1.lock().await,
            score_2: *self.score_2.lock().await,
            score_3: *self.score_3.lock().await,
            score_4: *self.score_4.lock().await,
        }
    }

    async fn try_send_update(&self) {
        let lock = self.tx.lock().await;
        if lock.is_some() {
            let mut tx = lock.as_ref().unwrap();

            if let Err(e) = tx.send(self.get_message().await).await {
                println!("Cannot update {e:?}");
            }
        }
    }
}

impl From<Msg> for Result<Event, Box<dyn std::error::Error + Send + Sync>> {
    fn from(value: Msg) -> Self {
        Event::default().json_data(value).map_err(Into::into)
    }
}

#[tokio::main]
async fn main() {
    let app = App::default();

    let state = Arc::new(app);

    use tower_http::cors::Any;
    let cors_layer = tower_http::cors::CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = axum::routing::Router::new()
        .route("/", get(health_check))
        .route("/1", post(add_1))
        .route("/2", post(add_2))
        .route("/3", post(add_3))
        .route("/4", post(add_4))
        .route("/IlllIllI", get(read))
        .route("/reset", post(reset))
        .with_state(state)
        .layer(cors_layer);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

read_n!(add_1, score_1);
read_n!(add_2, score_2);
read_n!(add_3, score_3);
read_n!(add_4, score_4);

async fn read(State(app): State<Arc<App>>) -> impl IntoResponse {
    let (tx, rx) = futures::channel::mpsc::unbounded();

    app.set_writer(tx).await;

    let stream = rx.map(Into::into);

    axum::response::sse::Sse::new(stream)
}

async fn reset(State(app): State<Arc<App>>, secret: TypedHeader<Authorization<Bearer>>) -> impl IntoResponse {
    if secret.token() != "P_LEON_KHOD_THEP" {
        return Err((StatusCode::FORBIDDEN, "Invalid token"));
    }

    *app.score_1.lock().await = 0;
    *app.score_2.lock().await = 0;
    *app.score_3.lock().await = 0;
    *app.score_4.lock().await = 0;
    app.try_send_update().await;

    Ok("OK")
}
