// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct RESTfulError {
    code: u16,
    err: anyhow::Error,
}

impl IntoResponse for RESTfulError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({
                "code": self.code,
                "message": self.err.to_string(),
            })),
        )
            .into_response()
    }
}

impl<E> From<E> for RESTfulError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            code: 500,
            err: err.into(),
        }
    }
}

pub async fn handle_http_error<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let response = next.run(req).await;
    let status_code = response.status();
    match status_code {
        StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR => response,
        _ => (
            status_code,
            Json(json!({
                "code": status_code.as_u16(),
                "message": status_code.canonical_reason().unwrap_or_default(),
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
pub struct RESTfulResponse<T: Serialize> {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T: Serialize> IntoResponse for RESTfulResponse<T> {
    fn into_response(self) -> Response {
        if let Some(data) = self.data {
            (
                StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(json!({
                    "code": self.code,
                    "message": self.message,
                    "data": data,
                })),
            )
                .into_response()
        } else {
            (
                StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(json!({
                    "code": self.code,
                    "message": self.message,
                })),
            )
                .into_response()
        }
    }
}

pub fn ok<T: Serialize>(data: T) -> Result<impl IntoResponse, RESTfulError> {
    Ok(RESTfulResponse {
        code: 200,
        message: "OK".to_string(),
        data: Some(data),
    })
}

pub fn ok_no_data() -> Result<impl IntoResponse, RESTfulError> {
    Ok(RESTfulResponse::<()> {
        code: 200,
        message: "OK".to_string(),
        data: None,
    })
}

pub fn err(code: u16, message: String) -> RESTfulError {
    RESTfulError {
        code,
        err: anyhow::anyhow!(message),
    }
}
