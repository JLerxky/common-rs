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

use std::fmt::{Display, Formatter};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use color_eyre::{eyre::Error, Result};
use serde::Serialize;
use serde_json::json;
use tokio::{net::TcpListener, signal};
use tracing::info;

use crate::error::CALError;

pub use axum;
pub use axum_extra;

#[derive(Debug)]
pub struct RESTfulError {
    pub code: u16,
    pub err: String,
}

impl Display for RESTfulError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "code: {}, message: {}", self.code, self.err)
    }
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
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        match err.into().downcast::<CALError>() {
            Ok(err) => Self {
                code: err.into(),
                err: err.to_string(),
            },
            Err(err) => Self {
                code: CALError::InternalServerError.into(),
                err: err.to_string(),
            },
        }
    }
}

async fn health() -> impl IntoResponse {
    ok_no_data()
}

pub async fn http_serve(service_name: &str, port: u16, router: Router) -> Result<()> {
    let router = router.route("/health", get(health));

    let listener = TcpListener::bind(format!("[::]:{}", port)).await?;

    info!(
        "{service_name} listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct RESTfulResponse<T: Serialize> {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

unsafe impl<T: Serialize> Send for RESTfulResponse<T> {}

impl<T: Serialize> IntoResponse for RESTfulResponse<T> {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            if let Some(data) = self.data {
                Json(json!({
                    "code": self.code,
                    "message": self.message,
                    "data": data,
                }))
            } else {
                Json(json!({
                    "code": self.code,
                    "message": self.message,
                }))
            },
        )
            .into_response()
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

pub fn err<W>(code: CALError, message: &str) -> Result<W, RESTfulError>
where
    W: IntoResponse,
{
    Err(RESTfulError {
        code: code.into(),
        err: message.to_owned(),
    })
}

pub fn err_code<W>(code: CALError) -> Result<W, RESTfulError>
where
    W: IntoResponse,
{
    Err(RESTfulError {
        code: code.into(),
        err: code.to_string(),
    })
}

pub fn err_msg<W>(message: &str) -> Result<W, RESTfulError>
where
    W: IntoResponse,
{
    Err(RESTfulError {
        code: CALError::InternalServerError.into(),
        err: message.to_owned(),
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("ctrl_c signal received"),
        _ = terminate => info!("terminate signal received"),
    }
}
