use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::prelude::*;
use clusterizer_common::{records::User, types::Id};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{query::SelectOne, state::AppState};

pub struct Auth(pub Id<User>);

pub enum AuthRejection {
    BadApiKey,
    UserDisabled,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        match self {
            Self::BadApiKey => (StatusCode::BAD_REQUEST, "Bad API Key provided").into_response(),
            Self::UserDisabled => (StatusCode::BAD_REQUEST, "User is disabled").into_response(),
        }
    }
}
impl FromRequestParts<AppState> for Auth {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>> = parts
            .extract()
            .await
            .map_err(|_| AuthRejection::BadApiKey)?;

        let mut api_key_bytes = [0; 40];
        let mut user_id_bytes = [0; 8];

        let length = BASE64_STANDARD
            .decode_slice(bearer.token(), &mut api_key_bytes)
            .map_err(|_| AuthRejection::BadApiKey)?;

        if length != api_key_bytes.len() {
            Err(AuthRejection::BadApiKey)?;
        }

        hmac(state, &api_key_bytes[..8])
            .verify_slice(&api_key_bytes[8..])
            .map_err(|_| AuthRejection::BadApiKey)?;

        user_id_bytes.copy_from_slice(&api_key_bytes[..8]);

        let user_id = i64::from_le_bytes(user_id_bytes).into();
        let user = User::select_one(user_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| AuthRejection::BadApiKey)?;

        if user.disabled_at.is_some() {
            Err(AuthRejection::UserDisabled)?;
        }

        Ok(Auth(user_id))
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
