//! Run with
//!
//! ```not_rust
//! cargo run -p example-sse
//! ```

use axum::{
    extract::{State, TypedHeader},
    response::sse::{Event, Sse},
    routing::get,
    Router,
};

use async_stream::stream;
use futures::stream::Stream;
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Context {
    counter: RwLock<usize>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let context = Arc::new(Context {
        counter: RwLock::new(0),
    });

    // build our application with a route
    let app = Router::new()
        .route("/sse", get(get_counter))
        .with_state(context.clone())
        .route("/increment", get(increment_counter))
        .with_state(context.clone());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_counter(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(context): State<Arc<Context>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let stream = stream! {
        for _ in 0.. {
            let data = context.counter.read().await.clone().to_string();
            yield Event::default().data(data);
        }
    }
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn increment_counter(State(context): State<Arc<Context>>) {
    let mut counter = context.counter.write().await;
    *counter += 1;
}
