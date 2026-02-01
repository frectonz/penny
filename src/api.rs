use axum::extract::{Query, State};
use axum::middleware;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use crate::auth::{auth_middleware, auth_status_handler};

use crate::reporter::{
    AppOverview, AppRun, PaginatedResponse, PaginationParams, Reporter, TimeRange, TotalOverview,
};
use crate::types::{Host, RunId};

#[derive(rust_embed::RustEmbed)]
#[folder = "ui/dist"]
pub struct UiAssets;

#[derive(Debug, Clone, Serialize)]
struct VersionResponse {
    version: &'static str,
}

async fn version_handler() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn static_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file first
    if let Some(content) = UiAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
            content.data.into_owned(),
        )
            .into_response();
    }

    // SPA fallback: serve index.html for all other routes
    match UiAssets::get("index.html") {
        Some(content) => (
            [(axum::http::header::CONTENT_TYPE, "text/html")],
            content.data.into_owned(),
        )
            .into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn total_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    Query(time_range): Query<TimeRange>,
) -> Json<TotalOverview> {
    let time_range = time_range.into_option();
    Json(reporter.total_overview(time_range).await)
}

async fn apps_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    Query(time_range): Query<TimeRange>,
) -> Json<Vec<AppOverview>> {
    let time_range = time_range.into_option();
    Json(reporter.apps_overview(time_range).await)
}

async fn app_overview_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(host): axum::extract::Path<String>,
    Query(time_range): Query<TimeRange>,
) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    let time_range = time_range.into_option();

    match reporter.app_overview(&Host(host), time_range).await {
        Some(overview) => Json(overview).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct AppRunsQuery {
    start: Option<i64>,
    end: Option<i64>,
    cursor: Option<i64>,
    limit: Option<u32>,
}

async fn app_runs_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(host): axum::extract::Path<String>,
    Query(query): Query<AppRunsQuery>,
) -> Json<PaginatedResponse<AppRun>> {
    let time_range = if query.start.is_some() || query.end.is_some() {
        Some(TimeRange {
            start: query.start,
            end: query.end,
        })
    } else {
        None
    };

    let pagination = PaginationParams {
        cursor: query.cursor,
        limit: query.limit,
    };

    Json(reporter.app_runs(&Host(host), time_range, pagination).await)
}

async fn run_logs_handler<R: Reporter>(
    State(reporter): State<R>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;

    match reporter.run_logs(&RunId::from_string(run_id)).await {
        Some(logs) => Json(logs).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn create_api_router<R: Reporter>(reporter: R) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new().route("/api/auth/status", get(auth_status_handler));

    // Protected routes (auth middleware applied)
    let protected_routes = Router::new()
        .route("/api/version", get(version_handler))
        .route("/api/total-overview", get(total_overview_handler::<R>))
        .route("/api/apps-overview", get(apps_overview_handler::<R>))
        .route("/api/app-overview/{host}", get(app_overview_handler::<R>))
        .route("/api/app-runs/{host}", get(app_runs_handler::<R>))
        .route("/api/run-logs/{run_id}", get(run_logs_handler::<R>))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(reporter);

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback(static_handler)
        .layer(cors)
}
