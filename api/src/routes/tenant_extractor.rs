use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::StatusCode;
use payroll_service::domain::ids::StandardID;
use payroll_service::domain::tenant::IDTenant;
use thiserror::Error;

#[derive(Debug)]
pub struct TenantId(pub StandardID<IDTenant>);

#[derive(Debug, Error)]
pub enum TenantIdRejection {
    #[error("Tenant ID is missing")]
    Missing,
    #[error("Tenant ID is invalid")]
    Invalid,
}

impl IntoResponse for TenantIdRejection {
    fn into_response(self) -> Response {
        match self {
            TenantIdRejection::Missing => {
                (StatusCode::BAD_REQUEST, "Tenant ID is missing").into_response()
            }
            TenantIdRejection::Invalid => {
                (StatusCode::BAD_REQUEST, "Tenant ID is invalid").into_response()
            }
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for TenantId {
    type Rejection = TenantIdRejection;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = match parts.headers.get("x-tenant-id") {
            Some(tenant_id) => {
                let raw = tenant_id.to_str().map_err(|_| TenantIdRejection::Invalid)?;
                StandardID::<IDTenant>::try_from(raw.to_string())
                    .map_err(|_| TenantIdRejection::Invalid)?
            }
            None => return Err(TenantIdRejection::Missing),
        };
        Ok(TenantId(tenant_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::extract::FromRequestParts;

    fn parts_with_header(value: &str) -> Parts {
        let (parts, _) = http::Request::builder()
            .header("x-tenant-id", value)
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
    async fn test_valid_tenant_id_is_extracted() {
        let mut parts = parts_with_header("000000000003V");
        let result = TenantId::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.to_string(), "000000000003V");
    }

    #[tokio::test]
    async fn test_missing_header_returns_missing_rejection() {
        let mut parts = parts_without_header();
        let result = TenantId::from_request_parts(&mut parts, &()).await;
        assert!(matches!(result.unwrap_err(), TenantIdRejection::Missing));
    }

    #[tokio::test]
    async fn test_invalid_id_returns_invalid_rejection() {
        let mut parts = parts_with_header("not-a-valid-tsid");
        let result = TenantId::from_request_parts(&mut parts, &()).await;
        assert!(matches!(result.unwrap_err(), TenantIdRejection::Invalid));
    }
}
