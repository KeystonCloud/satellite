use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};

use kc_core::{authentication::Claims, server::ServerState};

pub async fn handler(
    State(state): State<ServerState>,
    mut parts: Parts,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_schema = state.graphql_schema.clone();
    let mut request = req.into_inner().data(state.clone());

    let claims_result = Claims::from_request_parts(&mut parts, &state).await;
    if let Ok(claims_data) = claims_result {
        request = request.data(claims_data);
    }

    graphql_schema.execute(request).await.into()
}
