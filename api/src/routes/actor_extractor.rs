use axum::Json;
use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use http::request::Parts;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::user::IDUser;
use serde_json::json;
use thiserror::Error;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ActorId(pub StandardID<IDUser>);

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ActorIdRejection {
    #[error("Actor ID is missing")]
    Missing,
    #[error("Actor ID is invalid")]
    Invalid,
}

impl IntoResponse for ActorIdRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": self.to_string() })),
        )
            .into_response()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for ActorId {
    type Rejection = ActorIdRejection;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let actor_id = match parts.headers.get("x-actor-id") {
            Some(actor_id) => {
                let raw = actor_id.to_str().map_err(|_| ActorIdRejection::Invalid)?;
                StandardID::<IDUser>::try_from(raw.to_string())
                    .map_err(|_| ActorIdRejection::Invalid)?
            }
            None => return Err(ActorIdRejection::Missing),
        };
        Ok(ActorId(actor_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::extract::FromRequestParts;

    fn parts_with_header(value: &str) -> Parts {
        let (parts, _) = http::Request::builder()
            .header("x-actor-id", value)
            .body(())
            .unwrap()
            .into_parts();
        parts
    }

    fn parts_without_header() -> Parts {
        let (parts, _) = http::Request::builder().body(()).unwrap().into_parts();
        parts
    }

    #[tokio::test]
    async fn test_valid_actor_id_is_extracted() {
        let mut parts = parts_with_header("000000000003V");
        let result = ActorId::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.to_string(), "000000000003V");
    }

    #[tokio::test]
    async fn test_missing_header_returns_missing_rejection() {
        let mut parts = parts_without_header();
        let result = ActorId::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result.unwrap_err(), ActorIdRejection::Missing));
    }

    #[tokio::test]
    async fn test_invalid_actor_id_returns_invalid_rejection() {
        let mut parts = parts_with_header("not-a-valid-tsid");
        let result = ActorId::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result.unwrap_err(), ActorIdRejection::Invalid));
    }
}
