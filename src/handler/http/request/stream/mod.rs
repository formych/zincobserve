// Copyright 2022 Zinc Labs Inc. and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use actix_web::{get, http, post, web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;

use crate::common::http::get_stream_type_from_request;
use crate::meta::{self, stream::StreamSettings, StreamType};
use crate::service::stream;

#[utoipa::path(
    context_path = "/api",
    tag = "Streams",
    operation_id = "StreamSchema",
    security(
        ("Authorization"= [])
    ),
    params(
        ("org_id" = String, Path, description = "Organization name"),
        ("stream_name" = String, Path, description = "Stream name"),
    ),
    responses(
        (status = 200, description="Success", content_type = "application/json", body = Stream),
        (status = 400, description="Failure", content_type = "application/json", body = HttpResponse),
    )
)]
#[get("/{org_id}/{stream_name}/schema")]
async fn schema(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (org_id, stream_name) = path.into_inner();
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    let stream_type = match get_stream_type_from_request(&query) {
        Ok(v) => v,
        Err(e) => {
            return Ok(
                HttpResponse::BadRequest().json(meta::http::HttpResponse::error(
                    http::StatusCode::BAD_REQUEST.into(),
                    Some(e.to_string()),
                )),
            )
        }
    };
    match stream_type {
        Some(stream_type_loc) => {
            stream::get_stream(org_id.as_str(), stream_name.as_str(), stream_type_loc).await
        }
        /* Some(StreamType::Logs) => {
            stream::get_stream(org_id.as_str(), stream_name.as_str(), StreamType::Logs).await
        }
        Some(StreamType::Metrics) => {
            stream::get_stream(org_id.as_str(), stream_name.as_str(), StreamType::Metrics).await
        }
        Some(StreamType::Traces) => {
            stream::get_stream(org_id.as_str(), stream_name.as_str(), StreamType::Traces).await
        } */
        None => stream::get_stream(org_id.as_str(), stream_name.as_str(), StreamType::Logs).await,
    }
}

#[utoipa::path(
    context_path = "/api",
    tag = "Streams",
    operation_id = "StreamSettings",
    security(
        ("Authorization"= [])
    ),
    params(
        ("org_id" = String, Path, description = "Organization name"),
        ("stream_name" = String, Path, description = "Stream name"),
    ),
    request_body(content = StreamSettings, description = "Stream settings", content_type = "application/json"),
    responses(
        (status = 200, description="Success", content_type = "application/json", body = HttpResponse),
        (status = 400, description="Failure", content_type = "application/json", body = HttpResponse),
    )
)]
#[post("/{org_id}/{stream_name}/settings")]
async fn settings(
    path: web::Path<(String, String)>,
    settings: web::Json<StreamSettings>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (org_id, stream_name) = path.into_inner();
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    let stream_type = match get_stream_type_from_request(&query) {
        Ok(v) => v,
        Err(e) => {
            return Ok(
                HttpResponse::BadRequest().json(meta::http::HttpResponse::error(
                    http::StatusCode::BAD_REQUEST.into(),
                    Some(e.to_string()),
                )),
            )
        }
    };

    match stream_type {
        Some(steam_type_loc) => {
            stream::save_stream_settings(
                org_id.as_str(),
                stream_name.as_str(),
                steam_type_loc,
                settings.into_inner(),
            )
            .await
        }
        None => {
            stream::save_stream_settings(
                org_id.as_str(),
                stream_name.as_str(),
                StreamType::Logs,
                settings.into_inner(),
            )
            .await
        }
    }
}

#[utoipa::path(
    context_path = "/api",
    tag = "Streams",
    operation_id = "StreamList",
    security(
        ("Authorization"= [])
    ),
    params(
        ("org_id" = String, Path, description = "Organization name"),
    ),
    responses(
        (status = 200, description="Success", content_type = "application/json", body = ListStream),
        (status = 400, description="Failure", content_type = "application/json", body = HttpResponse),
    )
)]
#[get("/{org_id}/streams")]
async fn list(org_id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    let stream_type = match get_stream_type_from_request(&query) {
        Ok(v) => v,
        Err(e) => {
            return Ok(
                HttpResponse::BadRequest().json(meta::http::HttpResponse::error(
                    http::StatusCode::BAD_REQUEST.into(),
                    Some(e.to_string()),
                )),
            )
        }
    };

    let fetch_schema = match query.get("fetchSchema") {
        Some(s) => match s.to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    " 'fetchSchema' query param with value 'true' or 'false' allowed",
                ));
            }
        },
        None => false,
    };

    match stream_type {
        /* Some(StreamType::Logs) => {
            stream::list_streams(org_id.as_str(), Some(StreamType::Logs), fetch_schema).await
        }
        Some(StreamType::Metrics) => {
            stream::list_streams(org_id.as_str(), Some(StreamType::Metrics), fetch_schema).await
        }
        Some(StreamType::Traces) => {
            stream::list_streams(org_id.as_str(), Some(StreamType::Traces), fetch_schema).await
        } */
        Some(stream_type_loc) => {
            stream::list_streams(org_id.as_str(), Some(stream_type_loc), fetch_schema).await
        }
        None => stream::list_streams(org_id.as_str(), None, fetch_schema).await,
    }
}

#[get("/{org_id}/")]
async fn org_index(_org_id: web::Path<String>, req: HttpRequest) -> impl Responder {
    // eg.1: User-Agent:[elastic-transport-ruby/8.0.1 (RUBY_VERSION: 3.1.2; linux x86_64; Faraday v1.10.0)]
    let mut version = "0.0.0";
    let user_agent = match req.headers().get("User-Agent") {
        Some(user_agent) => user_agent.to_str().unwrap(),
        None => "",
    };
    if user_agent.contains("elastic") {
        let re = regex::Regex::new(r"(\d+\.\d+\.\d+)").unwrap();
        version = match re.captures(user_agent) {
            Some(caps) => caps.get(1).unwrap().as_str(),
            None => "8.1.0",
        };
    }
    let es_info = r##"{"name":"Elasticsearch","cluster_name":"N/A","cluster_uuid":"N/A","version":{"number":"0.0.0","build_flavor":"default","build_hash":"0","build_date":"0","build_snapshot":false,"lucene_version":"N/A","minimum_wire_version":"N/A","minimum_index_compatibility":"N/A"},"tagline":"You Know, for Search"}"##;
    let es_info = es_info.replace("0.0.0", version);
    HttpResponse::Ok()
        .content_type(http::header::ContentType::json())
        .insert_header(("X-Elastic-Product", "Elasticsearch"))
        .body(es_info)
}
