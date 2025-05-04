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
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{result::ApiError, state::AppState};

pub struct Auth(pub i64);

impl FromRequestParts<AppState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        let api_key_bytes = BASE64_STANDARD
            .decode(bearer.token())
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let id_bytes: [u8; 8] = api_key_bytes[0..8]
            .try_into()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Hmac::<Sha256>::new_from_slice(&state.secret)
            .unwrap()
            .chain_update(id_bytes)
            .verify_slice(&api_key_bytes[8..])
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(Auth(i64::from_le_bytes(id_bytes)))
    }
}

pub fn api_key(state: &AppState, id: i64) -> String {
    let id_bytes = id.to_le_bytes();
    let mac_bytes = Hmac::<Sha256>::new_from_slice(&state.secret)
        .unwrap()
        .chain_update(id_bytes)
        .finalize()
        .into_bytes();
    let api_key_bytes = [&id_bytes, &mac_bytes[..]].concat();

    BASE64_STANDARD.encode(&api_key_bytes)
}
