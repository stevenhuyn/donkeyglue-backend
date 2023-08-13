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
use futures::stream::{self, Map, RepeatWith, Stream};
use std::{convert::Infallible, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};
use tokio_stream::StreamExt;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Context {
    counter: RwLock<usize>,
}

struct Subscribers {
    counter_subscriber: Box<dyn Stream<Item = Result<Event, Infallible>> + Send + Sync + Unpin>,
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
        .route("/sse", get(sse_handler))
        .with_state(context.clone());

    // let stream: Throttle<Map<RepeatWith<impl Fn() -> Event>, Ok<Event, Infallible>(Event) -> Result<Event, Infallible>>> =
    //     stream::repeat_with(move || Event::default().data("test")).map(Ok).throttle(Duration::from_secs(1));

    let blah = stream::repeat_with(move || async { Event::default().data("test") });

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(context): State<Arc<Context>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // let count = std::sync::RwLock::new(3);
    // let count = tokio::sync::RwLock::new(3);

    // let stream = stream::repeat_with(move || Event::default().data("test"))
    //     .map(Ok)
    //     .throttle(Duration::from_secs(1));

    // let counter = Arc::new(RwLock::new(3));
    // let stream = stream::repeat_with({
    //     let counter = counter.clone();
    //     move || async move {
    //         let read_guard = counter.read().await;
    //         Ok::<_, Infallible>(*read_guard)
    //     }
    // });

    let counter = Arc::new(RwLock::new(3));
    let stream = stream! {
        for _ in 0.. {
            yield Event::default().data(counter.read().await.clone().to_string());
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
