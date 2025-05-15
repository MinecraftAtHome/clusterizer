use axum::{
    extract::{FromRequestParts, Path, State}, http::{request::Parts, StatusCode}, response::{IntoResponse, Response}, routing::get, RequestPartsExt
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::prelude::*;
use clusterizer_common::{id::Id, types::User};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{query::SelectOne, routes::{get_one, get_one_by}, state::AppState};

pub struct AuthRejection;

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, "Authorization failed").into_response()
    }
}

pub struct Auth(pub Id<User>);

impl FromRequestParts<AppState> for Auth {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>> =
            parts.extract().await.map_err(|_| AuthRejection)?;

        let mut api_key_bytes = [0; 40];
        let mut user_id_bytes = [0; 8];
        
        
        let length = BASE64_STANDARD
            .decode_slice(bearer.token(), &mut api_key_bytes)
            .map_err(|_| AuthRejection)?;

        if length != api_key_bytes.len() {
            Err(AuthRejection)?;
        }

        hmac(state, &api_key_bytes[..8])
            .verify_slice(&api_key_bytes[8..])
            .map_err(|_| AuthRejection)?;

        user_id_bytes.copy_from_slice(&api_key_bytes[..8]);
        let user_id = i64::from_le_bytes(user_id_bytes).into();
        let user = get_one::<User>(State(state.clone()), Path(user_id)).await;
        match user {
            Ok(user) => {
                if user.disabled_at.is_some(){
                    return Err(AuthRejection)
                }
            },
            Err(e) =>{
                return Err(AuthRejection)
            }
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
