use axum::body::Body;
use tracing::subscriber::set_global_default;
use tracing::{Span, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use axum::http::{HeaderName, Request};
use tower::ServiceBuilder;
use tower::layer::util::Stack;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::request_id::{MakeRequestId, RequestId, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tower_layer::Identity;
use uuid::Uuid;

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Create a tower_http Tracer Layer for the application
pub fn create_subscriber_middleware_layer() -> ServiceBuilder<
    Stack<
        // Note the exact types here:
        TraceLayer<SharedClassifier<ServerErrorsAsFailures>, fn(&Request<Body>) -> Span>,
        Stack<SetRequestIdLayer<MyMakeRequestId>, Identity>,
    >,
> {
    let x_request_id = HeaderName::from_static("x-request-id");

    ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(x_request_id, MyMakeRequestId))
        .layer(
            // Pass the function pointer here instead of a closure
            TraceLayer::new_for_http().make_span_with(make_span_with_request_id),
        )
}

#[derive(Clone, Copy)]
pub struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|id| id.header_value().to_str().unwrap_or(""))
        .unwrap_or("");

    tracing::info_span!(
        "http_request",
        %request_id,
        method = %request.method(),
        uri = %request.uri(),
    )
}
