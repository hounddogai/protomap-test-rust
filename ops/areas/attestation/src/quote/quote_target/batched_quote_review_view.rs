use super::*;

use acme_kit::text;

/// How screens and exports show a batched quote review.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub visible_text: String,
    pub rotation_days: String,
    pub currency: String,
    pub country_text: String,
    pub unit: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_batched_quote_review_view(item: &BatchedQuoteReview) -> BatchedQuoteReviewView {
    BatchedQuoteReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        summary: item.summary.clone(),
        visible_text: format!("{}", item.visible),
        rotation_days: item.rotation_days.to_string(),
        currency: text::truncate(&item.currency, 40),
        country_text: item.country.clone(),
        unit: item.unit.clone(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_quote_review_views(items: &[BatchedQuoteReview]) -> Vec<BatchedQuoteReviewView> {
    let mut views: Vec<BatchedQuoteReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_quote_review_view(item));
    }
    views
}

pub fn sum_batched_quote_review_view(view: &BatchedQuoteReviewView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_quote_review("code-575", "label-966");
        let view = to_batched_quote_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_quote_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
