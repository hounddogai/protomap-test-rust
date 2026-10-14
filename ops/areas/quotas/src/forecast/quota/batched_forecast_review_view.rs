use super::*;

use acme_kit::text;

/// How screens and exports show a batched forecast review.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub scope_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_batched_forecast_review_view(item: &BatchedForecastReview) -> BatchedForecastReviewView {
    BatchedForecastReviewView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        quantity: item.quantity.to_string(),
        scope_text: text::truncate(&item.scope, 64),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_forecast_review_views(items: &[BatchedForecastReview]) -> Vec<BatchedForecastReviewView> {
    let mut views: Vec<BatchedForecastReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_forecast_review_view(item));
    }
    views
}

pub fn sum_batched_forecast_review_view(view: &BatchedForecastReviewView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_forecast_review("code-213", "name-61");
        let view = to_batched_forecast_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_forecast_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
