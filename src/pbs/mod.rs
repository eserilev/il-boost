use axum::{async_trait, body::Body, extract::State, http::{HeaderMap, Response}, response::IntoResponse, routing::{get, post}, Json, Router};
use cb_pbs::{BuilderApi, BuilderApiStat, PbsState};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::inclusion_boost::types::InclusionList;

// Any method that is not overriden will default to the normal MEV boost flow
pub struct InclusionBoostApi;


#[derive(Debug, Clone, Default, Deserialize)]
pub struct InclusionBoost;


impl BuilderApiState for InclusionBoost {}

#[async_trait]
impl BuilderApi<InclusionBoost> for InclusionBoostApi {

    fn extra_routes() -> Option<Router<PbsState<InclusionBoost>>> {
        let router = Router::new()
            .route("/custom/stats", post(handle_post_constraints));
        
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
    State(state): State<PbsState<InclusionBoost>>,
    _: HeaderMap,
    Json(inclusion_list): Json<InclusionList>
) -> Response<Body> {
    let response = state.relay_client()
        .post("url")
        .json(&inclusion_list)
        .send()
        .await
        .unwrap();

    let response_body = Body::from(response.bytes().await.unwrap());
    (StatusCode::OK, response_body).into_response()
}