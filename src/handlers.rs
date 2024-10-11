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
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i32;
    let offset = ((opts.page.unwrap_or(1) - 1) * limit as usize) as i32;

    let query_result = sqlx::query_as!(
        Feedback,
        r#"SELECT * FROM feedbacks ORDER BY id LIMIT ? OFFSET ?"#,
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
    let feedback_id = uuid::Uuid::new_v4().to_string();
    let query_result = sqlx::query(
        r#"INSERT INTO feedbacks (id,name,email,feedback,rating) VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(feedback_id.clone())
    .bind(body.name.to_string())
    .bind(body.email.to_string())
    .bind(body.feedback.to_string())
    .bind(body.rating)
    .execute(&data.db)
    .await
    .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Feedback already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    let feedback = sqlx::query_as!(
        Feedback,
        r#"SELECT * FROM feedbacks WHERE id = ?"#,
        feedback_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    let feedback_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "feedback": feedback
    })});

    Ok(Json(feedback_response))
}

pub async fn get_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        Feedback,
        "SELECT * FROM feedbacks WHERE id = ?",
        id.to_string()
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(feedback) => {
            let feedback_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "feedback": feedback
            })});

            return Ok(Json(feedback_response));
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Feedback with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn edit_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateFeedbackSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        Feedback,
        r#"SELECT * FROM feedbacks WHERE id = ?"#,
        id.to_string()
    )
    .fetch_one(&data.db)
    .await;

    let feedback = match query_result {
        Ok(feedback) => feedback,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Feedback with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };

    let now = chrono::Utc::now();

    let update_query = sqlx::query(
        r#"UPDATE feedbacks SET name = ?, email = ?, feedback = ?, rating = ?, updated_at = ? WHERE id = ?"#,
    ).bind(body.name.to_owned().unwrap_or(feedback.name))
    .bind(body.email.to_owned().unwrap_or(feedback.email))
    .bind(body.feedback.to_owned().unwrap_or(feedback.feedback))
    .bind(body.rating.unwrap_or(feedback.rating))
    .bind(now)
    .bind(id.to_string())
    .execute(&data.db)
    .await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    if update_query.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Feedback with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let updated_feedback = sqlx::query_as!(
        Feedback,
        r#"SELECT * FROM feedbacks WHERE id = ?"#,
        id.to_string()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    let feedback_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "feedback": updated_feedback
    })});

    Ok(Json(feedback_response))
}

pub async fn delete_feedback_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query!("DELETE FROM feedbacks  WHERE id = ?", id.to_string())
        .execute(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    if query.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Feedback with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}
