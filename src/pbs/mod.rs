// use std::marker::PhantomData;

// use axum::{async_trait, body::Body, extract::State, http::{HeaderMap, Response}, response::IntoResponse, routing::{get, post}, Router};
// use cb_pbs::{BuilderApi, BuilderApiState, GetHeaderParams, GetHeaderReponse, PbsState};
// use reqwest::StatusCode;


// // Any method that is not overriden will default to the normal MEV boost flow
// struct InclusionBoostApi;


// #[derive(Debug, Clone, Default)]
// struct InclusionBoost;


// impl BuilderApiState for InclusionBoost {}


// impl BuilderApi<InclusionBoost> for InclusionBoostApi {

//     fn extra_routes() -> Option<Router<PbsState<InclusionBoost>>> {
//         let router = Router::new().route("/custom/stats", post(handle_post_constraints));
//         Some(router)
//     }

    
//     // https://ethereum.github.io/builder-specs/#/Builder/getHeader
//     // async fn get_header,(
//     //     params: GetHeaderParams,
//     //     req_headers: HeaderMap,
//     //     state: PbsState<InclusionBoost>,
//     // ) { // eyre::Result<Option<GetHeaderReponse>> {
//     //     todo!()
//     //     // mev_boost::get_header(params, req_headers, state).await
//     // }
// }

// fn handle_post_constraints(State(state): State<PbsState<InclusionBoost>>) -> Response<Body> {
//     (StatusCode::OK, ()).into_response()
// }