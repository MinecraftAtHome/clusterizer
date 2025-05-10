use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::prelude::*;
use clusterizer_common::{id::Id, types::User};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{result::ApiError, state::AppState};

pub struct Auth(pub Id<User>);

impl FromRequestParts<AppState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut api_key_bytes = [0; 40];
        let mut user_id_bytes = [0; 8];

        let length = BASE64_STANDARD
            .decode_slice(bearer.token(), &mut api_key_bytes)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        if length != api_key_bytes.len() {
            Err(StatusCode::BAD_REQUEST)?;
        }

        hmac(state, &api_key_bytes[..8])
            .verify_slice(&api_key_bytes[8..])
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        user_id_bytes.copy_from_slice(&api_key_bytes[..8]);

        Ok(Auth(i64::from_le_bytes(user_id_bytes).into()))
    }
}

pub fn api_key(state: &AppState, user_id: Id<User>) -> String {
    let user_id_bytes = user_id.raw().to_le_bytes();
    let hmac_bytes = hmac(state, &user_id_bytes).finalize().into_bytes();
    let mut api_key_bytes = [0; 40];

    api_key_bytes[..8].copy_from_slice(&user_id_bytes);
    api_key_bytes[8..].copy_from_slice(&hmac_bytes);

    BASE64_STANDARD.encode(api_key_bytes)
}

fn hmac(state: &AppState, bytes: &[u8]) -> Hmac<Sha256> {
    Hmac::new_from_slice(&state.secret)
        .unwrap()
        .chain_update(bytes)
}
