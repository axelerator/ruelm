use axum::{
    extract::{self, Path, State},
    response::{sse::Event, Sse},
    routing::{get, post},
    Form, Router,
};

use axum::response::Redirect;
use elm_rs::{Elm, ElmDecode, ElmEncode};
use futures::stream::Stream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, sync::Arc};
use std::{convert::Infallible, time::Duration};
use tokio::sync::{mpsc::Sender, RwLock};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

type SessionId = Uuid;

struct Clients {
    connections: HashMap<SessionId, Sender<ToFrontend>>,
    authenticated: HashMap<SessionId, String>,
    worker: Sender<WorkerMessage>,
}

impl Clients {
    async fn send(&self, session_id: SessionId, message: ToFrontend) {
        if let Some(sender) = self.connections.get(&session_id) {
            sender.send(message).await.unwrap();
        }
    }
}

impl Clients {
    fn try_login(&mut self, username: &str, password: &str) -> Option<SessionId> {
        if password == "pw" {
            // replace with proper password matching
            let session_id = Uuid::new_v4();
            self.authenticated.insert(session_id, username.to_string());
            Some(session_id)
        } else {
            None
        }
    }
}

type SharedState = Arc<RwLock<Clients>>;

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode)]
enum ToBackend {
    Connect,
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmDecode)]
enum ToFrontend {
    Welcome(String),
    SessionExpired,
}

enum WorkerMessage {
    FromFrontend(SessionId, ToBackend),
}

#[tokio::main]
async fn main() {
    write_elm_types();
    let www_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("www");
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<WorkerMessage>(32);

    let clients = Clients {
        connections: HashMap::new(),
        authenticated: HashMap::new(),
        worker: sender,
    };
    let shared_state = Arc::new(RwLock::new(clients));
    let state_for_worker = shared_state.clone();

    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                WorkerMessage::FromFrontend(session_id, message) => match message {
                    ToBackend::Connect => {
                        let clients = state_for_worker.read().await;
                        clients
                            .send(
                                session_id,
                                ToFrontend::Welcome("Hello from Rust".to_string()),
                            )
                            .await;
                    }
                },
            }
        }
    });

    let app = Router::new()
        .route_service("/", ServeFile::new(www_dir.join("index.html")))
        .route("/login", post(handle_login))
        .route("/sse/:session_id", get(sse_handler))
        .route("/send/:session_id", post(handle_send))
        .nest_service("/assets", ServeDir::new(www_dir.join("assets")))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Debug)]
struct SignUp {
    username: String,
    password: String,
}
async fn handle_send(
    Path(session_id): Path<SessionId>,
    State(state): State<SharedState>,
    extract::Json(to_backend): extract::Json<ToBackend>,
) {
    let _ = state
        .read()
        .await
        .worker
        .send(WorkerMessage::FromFrontend(session_id, to_backend))
        .await;
}

async fn handle_login(State(state): State<SharedState>, Form(login): Form<SignUp>) -> Redirect {
    if let Some(session_id) = state
        .write()
        .await
        .try_login(&login.username, &login.password)
    {
        Redirect::to(format!("/?session_id={}", session_id).as_str())
    } else {
        Redirect::to("/?login_failed=1")
    }
}

async fn sse_handler(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (sender, receiver) = tokio::sync::mpsc::channel::<ToFrontend>(32);
    let stream = tokio_stream::wrappers::ReceiverStream::new(receiver)
        .map(|to_frontend| Ok(Event::default().json_data(to_frontend).unwrap()));

    let mut clients = state.write().await;
    if clients.authenticated.contains_key(&session_id) {
        clients.connections.insert(session_id, sender);
    } else {
        sender.send(ToFrontend::SessionExpired).await.unwrap();
    };
    drop(clients);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

fn write_elm_types() {
    // the target would typically be a file
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!("Generated.Bindings", &mut target, {
        // generates types and encoders for types implementing ElmEncoder
        encoders: [ ToBackend ],
        // generates types and decoders for types implementing ElmDecoder
        decoders: [ ToFrontend ],
        // generates types and functions for forming queries for types implementing ElmQuery
        queries: [],
        // generates types and functions for forming queries for types implementing ElmQueryField
        query_fields: [],
    })
    .unwrap();
    let target_filename =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("client/src/generated/Bindings.elm");
    File::create(target_filename)
        .unwrap()
        .write_all(String::from_utf8(target).unwrap().as_bytes())
        .unwrap();
}
