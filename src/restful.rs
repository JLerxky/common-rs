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

use color_eyre::eyre::{eyre, Error};
use salvo::{catcher::Catcher, prelude::*};
use serde::Serialize;
use serde_json::json;
use tokio::signal;

#[derive(Debug)]
pub struct RESTfulError {
    code: u16,
    err: Error,
}

#[async_trait]
impl Writer for RESTfulError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code(
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
        res.render(Json(json!({
            "code": self.code,
            "message": self.err.to_string(),
        })));
    }
}

impl<E> From<E> for RESTfulError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self {
            code: 500,
            err: err.into(),
        }
    }
}

#[handler]
async fn handle_http_error(
    &self,
    _req: &Request,
    _depot: &Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if let Some(status_code) = res.status_code {
        match status_code {
            StatusCode::OK | StatusCode::INTERNAL_SERVER_ERROR => {}
            _ => {
                res.render(Json(json!({
                    "code": status_code.as_u16(),
                    "message": status_code.canonical_reason().unwrap_or_default(),
                })));
            }
        }
        ctrl.skip_rest();
    }
}

#[handler]
async fn health() -> impl Writer {
    ok_no_data()
}

pub async fn http_serve(service_name: &str, port: u16, router: Router) {
    let router = router.push(Router::with_path("health").get(health));

    let doc = OpenApi::new(format!("{} api", service_name), "0.0.1").merge_router(&router);

    let router = router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"));

    let service = Service::new(router).catcher(Catcher::default().hoop(handle_http_error));

    let acceptor = TcpListener::new(format!("0.0.0.0:{}", port)).bind().await;

    Server::new(acceptor)
        .serve_with_graceful_shutdown(service, async move { shutdown_signal().await }, None)
        .await;
    info!("{service_name} listening on 0.0.0.0:{port}");
}

#[derive(Debug, Serialize)]
pub struct RESTfulResponse<T: Serialize> {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

unsafe impl<T: Serialize> Send for RESTfulResponse<T> {}

#[async_trait]
impl<T: Serialize> Writer for RESTfulResponse<T> {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code(
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
        if let Some(data) = self.data {
            res.render(Json(json!({
                "code": self.code,
                "message": self.message,
                "data": data,
            })));
        } else {
            res.render(Json(json!({
                "code": self.code,
                "message": self.message,
            })));
        }
    }
}

pub fn ok<T: Serialize>(data: T) -> Result<impl Writer, RESTfulError> {
    Ok(RESTfulResponse {
        code: 200,
        message: "OK".to_string(),
        data: Some(data),
    })
}

pub fn ok_no_data() -> Result<impl Writer, RESTfulError> {
    Ok(RESTfulResponse::<()> {
        code: 200,
        message: "OK".to_string(),
        data: None,
    })
}

pub fn err(code: u16, message: String) -> RESTfulError {
    RESTfulError {
        code,
        err: eyre!(message),
    }
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}
