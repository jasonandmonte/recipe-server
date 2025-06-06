// From https://github.com/shuttle-hq/shuttle-examples/axum/jwt-authentication

use crate::*;

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// Read secret from file from given env path
pub async fn read_secret(env_var: &str, default: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret_file = std::env::var(env_var).unwrap_or_else(|_| default.to_owned());
    let secret = tokio::fs::read_to_string(secret_file).await?;
    Ok(secret.trim().to_string())
}

pub async fn make_jwt_keys() -> Result<JwtKeys, Box<dyn std::error::Error>> {
    let secret = read_secret("JWT_SECRETFILE", "secrets/jwt_secret.txt").await?;
    Ok(JwtKeys::new(secret.as_bytes()))
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Internal error: token creation")]
    TokenCreation,
    #[error("Registration error")]
    Registration,
}

impl utoipa::PartialSchema for AuthError {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
        serde_json::json!({
            "status":"401","error":"Invalid Credentials"
        }).into()
    }
}

impl utoipa::ToSchema for AuthError {}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

impl IntoResponse for AuthBody {
    fn into_response(self) -> axum::response::Response {
        Json(serde_json::json!(self)).into_response()
    }
}

impl axum::extract::FromRequestParts<SharedAppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut http::request::Parts, state: &SharedAppState) -> Result<Self, Self::Rejection> {
        use jsonwebtoken::{Algorithm, Validation, decode};

        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let app_state = state.read().await;
        let decoding_key = &app_state.jwt_keys.decoding;
        let validation = Validation::new(Algorithm::HS512);
        let result = decode::<Claims>(
            bearer.token(),
            decoding_key,
            &validation,
        );
        let token_data = result.map_err(|_| AuthError::Registration)?;
        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::Registration => (StatusCode::UNAUTHORIZED, "Invalid registration"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        let body = Json(serde_json::json!({
            "status": status.as_u16(),
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Registration {
    #[schema(example = "First Last")]
    full_name: String,
    #[schema(example = "firstlast@example.com")]
    email: String,
    #[schema(example = "openinvite")]
    access_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = "recipe-server.po8.org")]
    iss: String,
    #[schema(example = "First Last <firstlast@example.com>")]
    sub: String,
    #[schema(example = "1764662400")] // Unix timestamp
    exp: u64,
}

pub fn make_jwt_token(app_state: &AppState, registration: &Registration) -> Result<AuthBody, AuthError> {
    use jsonwebtoken::{Algorithm, Header, encode};
    
    if registration.access_code != app_state.reg_key {
        return Err(AuthError::Registration);
    }

    let iss = "recipe-server.po8.org".to_string();
    let sub = format!("{} <{}>", registration.full_name, registration.email);
    let exp = (Utc::now() + TimeDelta::days(1)).timestamp();
    let exp = u64::try_from(exp).unwrap();
    let claims = Claims { iss, sub, exp };
    let header = Header::new(Algorithm::HS512);
    let token = encode(&header, &claims, &app_state.jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(AuthBody::new(token))
}
