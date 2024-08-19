use std::marker::PhantomData;

use axum::{
    async_trait,
    body::Body,
    extract::State,
    http::{HeaderMap, Response},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use cb_pbs::{BuilderApi, BuilderApiState, PbsState};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{config::InclusionListConfig, inclusion_boost::types::InclusionList};

// Any method that is not overriden will default to the normal MEV boost flow
pub struct InclusionBoostApi;

impl BuilderApiState for InclusionListConfig {}

#[async_trait]
impl BuilderApi<InclusionListConfig> for InclusionBoostApi {
    fn extra_routes() -> Option<Router<PbsState<InclusionListConfig>>> {
        let router = Router::new().route("/constraints", post(handle_post_constraints));

        Some(router)
    }

    // fn get_header(
    //     params: GetHeaderParams,
    //     req_headers: HeaderMap,
    //     state: PbsState<InclusionBoost>,
    // ) -> Result<Option<GetHeaderReponse>, ()> {
    //     todo!()
    // }
}

async fn handle_post_constraints(
    State(state): State<PbsState<InclusionListConfig>>,
    _: HeaderMap,
    Json(inclusion_list): Json<InclusionList>,
) -> Response<Body> {
    // TODO unwrap
    let response = state
        .relays()
        .first()
        .unwrap()
        .client
        .post("test")
        .json(&inclusion_list)
        .send()
        .await
        .unwrap();

    let response_body = Body::from(response.bytes().await.unwrap());
    (StatusCode::OK, response_body).into_response()
}
