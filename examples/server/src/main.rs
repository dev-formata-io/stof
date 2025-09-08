use std::{net::SocketAddr, sync::Arc};
use axum::{extract::{Path, State}, response::IntoResponse, routing::any, Router};
use bytes::Bytes;
use rustc_hash::FxHashSet;
use stof::{model::Graph, runtime::Runtime};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut graph = Graph::default();
    if let Err(err) = graph.file_import("stof", "src/api.stof", None) {
        panic!("{}", err.to_string());
    }
    graph.remove_lib(&"fs".into()); // users of our api cannot interact with our filesystem now

    let app = Router::new()
        .route("/{attribute}", any(handler))
        .with_state(StofState { graph: Arc::new(Mutex::new(graph)) });

    println!("listening on 127.0.0.1:3030 (http://localhost:3030/{{attribute_handler}})");

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3030))).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

#[derive(Clone)]
pub struct StofState {
    pub graph: Arc<Mutex<Graph>>,
}

/// Parses a Stof body into the graph, runs #[attribute] functions, then exports a "Response" root as TOML.
/// In real life, use Content-Type header to parse data into graphs, and maybe an export query param for optional export format specification.
/// Another hint - keep your Stof API separate from stored data (ofc), this is just an example.
/// Use Stof library control and sandboxing to set what your users can do!
async fn handler(State(state): State<StofState>, Path(attribute): Path<String>, body: Bytes) -> impl IntoResponse {
    let mut graph = state.graph.lock().await;
    
    let response_root = graph.insert_root("Response");
    let request_root = graph.insert_root("Request");
    if let Err(error) = graph.binary_import("stof", body, Some(request_root.clone())) {
        graph.remove_node(&response_root, false);
        graph.remove_node(&request_root, false);
        return error.to_string();
    }

    let response_string;
    let mut attributes = FxHashSet::default();
    attributes.insert(attribute);
    match Runtime::run_attribute_functions(&mut graph, None, &Some(attributes), true) {
        Ok(_) => {
            if let Ok(response) = graph.string_export("toml", Some(response_root.clone())) {
                response_string = response;
            } else {
                response_string = "error exporting TOML Response data".into();
            }
        },
        Err(error) => {
            response_string = error;
        }
    }

    graph.remove_node(&request_root, false);
    graph.remove_node(&response_root, false);
    response_string
}
