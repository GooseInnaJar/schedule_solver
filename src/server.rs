use axum::{routing::post, Router, Json};
use crate::data::{SchedulingInput, SchedulingOutput};
use crate::solver;

async fn solve_handler(Json(input): Json<SchedulingInput>) -> Result<Json<SchedulingOutput>, (axum::http::StatusCode, String)> {
    match solver::solve(&input) {
        Ok(output) => Ok(Json(output)),
        Err(e) => Err((axum::http::StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn run_server() {
    let app = Router::new()
        .route("/v1/schedule/solve", post(solve_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Server running at http://{}", listener.local_addr().unwrap());
    
    axum::serve(listener, app).await.unwrap();
}
