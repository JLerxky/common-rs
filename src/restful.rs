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
use serde_json::json;

#[derive(Debug)]
pub struct RESTfulError {
    code: StatusCode,
    err: anyhow::Error,
}

impl IntoResponse for RESTfulError {
    fn into_response(self) -> Response {
        (
            self.code,
            Json(json!({
                "code": self.code.as_u16(),
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
            code: StatusCode::INTERNAL_SERVER_ERROR,
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

pub fn ok<T: serde::Serialize>(data: T) -> Result<impl IntoResponse, RESTfulError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "code": 200,
            "message": "OK",
            "data": data,
        })),
    ))
}

pub fn ok_no_data() -> Result<impl IntoResponse, RESTfulError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "code": 200,
            "message": "OK",
        })),
    ))
}

pub fn err(code: StatusCode, message: &str) -> Result<impl IntoResponse, RESTfulError> {
    Ok((
        code,
        Json(json!({
            "code": code.as_u16(),
            "message": message,
        })),
    ))
}
