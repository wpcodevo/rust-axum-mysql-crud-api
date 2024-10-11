use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::Feedback,
    schema::{CreateFeedbackSchema, FilterOptions, UpdateFeedbackSchema},
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Feedback CRUD API with Rust, SQLX, Postgres,and Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn feedback_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Unwrapping opts or using default if None
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let offset = ((opts.page.unwrap_or(1) - 1) * limit as usize) as i64;

    let query_result = sqlx::query_as!(
        Feedback,
        "SELECT * FROM feedbacks ORDER BY id LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&data.db)
    .await;

    match query_result {
        Ok(feedbacks) => {
            let json_response = serde_json::json!({
                "status": "success",
                "results": feedbacks.len(),
                "feedbacks": feedbacks
            });
            Ok(Json(json_response))
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something went wrong while fetching feedbacks",
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn create_feedback_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateFeedbackSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        Feedback,
        "INSERT INTO feedbacks (name,email,feedback,rating) VALUES ($1, $2, $3, $4) RETURNING *",
        body.name.to_string(),
        body.email.to_string(),
        body.feedback.to_string(),
        body.rating
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(feedback) => {
            let feedback_response = json!({"status": "success","data": json!({
                "feedback": feedback
            })});

            return Ok((StatusCode::CREATED, Json(feedback_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "This feedback already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn get_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(Feedback, "SELECT * FROM feedbacks WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(feedback) => {
            let feedback_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "feedback": feedback
            })});

            return Ok(Json(feedback_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Feedback with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

pub async fn edit_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateFeedbackSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(Feedback, "SELECT * FROM feedbacks WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Feedback with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let feedback = query_result.unwrap();

    let query_result = sqlx::query_as!(
        Feedback,
        "UPDATE feedbacks SET name = $1, email = $2, feedback = $3, rating = $4, updated_at = $5 WHERE id = $6 RETURNING *",
        body.name.to_owned().unwrap_or(feedback.name),
        body.email.to_owned().unwrap_or(feedback.email),
        body.feedback.to_owned().unwrap_or(feedback.feedback),
        body.rating.unwrap_or(feedback.rating),
        now,
        id
    )
    .fetch_one(&data.db)
    .await
    ;

    match query_result {
        Ok(feedback) => {
            let feedback_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "feedback": feedback
            })});

            return Ok(Json(feedback_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}

pub async fn delete_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query!("DELETE FROM feedbacks  WHERE id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Feedback with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}
