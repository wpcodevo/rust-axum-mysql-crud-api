use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

// #[derive(Deserialize, Debug)]
// pub struct ParamOptions {
//     pub id: String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFeedbackSchema {
    pub name: String,
    pub email: String,
    pub feedback: String,
    pub rating: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateFeedbackSchema {
    pub name: Option<String>,
    pub email: Option<String>,
    pub feedback: Option<String>,
    pub rating: Option<f32>,
    pub status: Option<String>,
}
