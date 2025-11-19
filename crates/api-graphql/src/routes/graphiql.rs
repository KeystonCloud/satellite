use async_graphql::http::GraphiQLSource;
use axum::response::{Html, IntoResponse};

pub async fn handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/api/graphql").finish())
}
