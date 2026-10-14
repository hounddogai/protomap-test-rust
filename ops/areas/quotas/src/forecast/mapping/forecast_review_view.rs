use super::*;

use acme_kit::text;

/// How screens and exports show a forecast review.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReviewView {
    pub code: String,
    pub label_text: String,
    pub notes: String,
    pub scope: String,
    pub cluster_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_forecast_review_view(item: &ForecastReview) -> ForecastReviewView {
    ForecastReviewView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        notes: item.notes.clone(),
        scope: item.scope.clone(),
        cluster_text: item.cluster.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_review_views(items: &[ForecastReview]) -> Vec<ForecastReviewView> {
    let mut views: Vec<ForecastReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_review_view(item));
    }
    views
}

pub fn sum_forecast_review_view(view: &ForecastReviewView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
