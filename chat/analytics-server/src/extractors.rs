use axum::{
    async_trait,
    body::Body,
    extract::{FromRequest, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;

use crate::pb::GeoLocation;

pub struct Protobuf<T>(pub T);
pub struct Geo(pub Option<GeoLocation>);

#[allow(unused)]
pub enum ProtobufRejection {
    ProtobufDecodeError(prost::DecodeError),
    FailedToBufferBody,
    MissingProtobufContentType,
}

impl IntoResponse for ProtobufRejection {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match self {
            Self::ProtobufDecodeError(_) => (StatusCode::BAD_REQUEST, "Protobuf decode error"),
            Self::FailedToBufferBody => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error buffering request body",
            ),
            Self::MissingProtobufContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Missing 'content-type: application/protobuf' header",
            ),
        };

        Response::builder()
            .status(status)
            .body(Body::from(body))
            .unwrap() // we know this can't fail since we made it
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Protobuf<T>
where
    T: prost::Message + Default,
    S: Send + Sync,
{
    type Rejection = ProtobufRejection;

    async fn from_request(req: axum::extract::Request, _: &S) -> Result<Self, Self::Rejection> {
        req.headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("application/protobuf"))
            .ok_or(ProtobufRejection::MissingProtobufContentType)?;

        let mut body = req.into_body().into_data_stream();
        let mut buf = Vec::new();

        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|_| ProtobufRejection::FailedToBufferBody)?;
            buf.extend_from_slice(&chunk);
        }

        T::decode(buf.as_slice())
            .map(Self)
            .map_err(ProtobufRejection::ProtobufDecodeError)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Geo
where
    S: Send + Sync,
{
    type Rejection = ProtobufRejection;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let country = get_header_value(parts, "x-country");
        let city = get_header_value(parts, "x-city");
        let region = get_header_value(parts, "x-region");
        Ok(Self(match (country, city, region) {
            (Some(country), Some(city), Some(region)) => Some(GeoLocation {
                country,
                city,
                region,
            }),
            _ => None,
        }))
    }
}

fn get_header_value(parts: &Parts, name: &str) -> Option<String> {
    parts
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|v| v.to_string())
}
