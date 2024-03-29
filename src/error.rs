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

use num_enum::IntoPrimitive;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error, IntoPrimitive)]
#[repr(u16)]
pub enum CALError {
    #[error("Bad Request")]
    BadRequest = 400,
    #[error("Unauthorized")]
    Unauthorized = 401,
    #[error("Forbidden")]
    Forbidden = 403,
    #[error("Not Found")]
    NotFound = 404,
    #[error("Too Many Requests")]
    TooManyRequests = 429,
    #[error("Internal Server Error")]
    InternalServerError = 500,
    #[error("Not Implemented")]
    NotImplemented = 501,
    #[error("Bad Gateway")]
    BadGateway = 502,
    #[error("Service Unavailable")]
    ServiceUnavailable = 503,
    #[error("Gateway Timeout")]
    GatewayTimeout = 504,

    #[error("Chain Error")]
    ChainError = 1000,

    #[error("Transaction Error")]
    TransactionError = 2000,
    #[error("Transaction Timeout")]
    TransactionTimeout = 2001,
    #[error("Transaction Reverted")]
    TransactionReverted = 2002,

    #[error("KMS Error")]
    KMSError = 3000,

    #[error("External Error")]
    ExternalError = 4000,
    #[error("Cita CMC Create Failed")]
    CitaCMCCreateFailed = 4001,
}
