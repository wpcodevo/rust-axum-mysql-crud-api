use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::{
        create_feedback_handler, delete_feedback_handler, edit_feedback_handler, get_feedback_handler,
        health_checker_handler, feedback_list_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/feedbacks/", post(create_feedback_handler))
        .route("/api/feedbacks", get(feedback_list_handler))
        .route(
            "/api/feedbacks/:id",
            get(get_feedback_handler)
                .patch(edit_feedback_handler)
                .delete(delete_feedback_handler),
        )
        .with_state(app_state)
}
