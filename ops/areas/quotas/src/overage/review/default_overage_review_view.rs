use super::*;

use acme_kit::text;

/// How screens and exports show a default overage review.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub key_id_text: String,
    pub window_days: String,
    pub source: String,
    pub threshold: String,
    pub visible: String,
    pub limit: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_default_overage_review_view(item: &DefaultOverageReview) -> DefaultOverageReviewView {
    DefaultOverageReviewView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        category: item.category.clone(),
        key_id_text: item.key_id.clone(),
        window_days: item.window_days.to_string(),
        source: item.source.clone(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        limit: item.limit.to_string(),
        owner_id: text::truncate(&item.owner_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_overage_review_views(items: &[DefaultOverageReview]) -> Vec<DefaultOverageReviewView> {
    let mut views: Vec<DefaultOverageReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_overage_review_view(item));
    }
    views
}

pub fn sum_default_overage_review_view(view: &DefaultOverageReviewView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_threshold + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_overage_review("code-856", "title-701");
        let view = to_default_overage_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_overage_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
