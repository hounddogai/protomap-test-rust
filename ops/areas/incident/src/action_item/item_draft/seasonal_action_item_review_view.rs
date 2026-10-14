use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal action item review.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemReviewView {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_seasonal_action_item_review_view(item: &SeasonalActionItemReview) -> SeasonalActionItemReviewView {
    SeasonalActionItemReviewView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 120),
        tenant_id: item.tenant_id.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_action_item_review_views(items: &[SeasonalActionItemReview]) -> Vec<SeasonalActionItemReviewView> {
    let mut views: Vec<SeasonalActionItemReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_action_item_review_view(item));
    }
    views
}

pub fn sum_seasonal_action_item_review_view(view: &SeasonalActionItemReviewView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_action_item_review("code-187", "label-445");
        let view = to_seasonal_action_item_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_action_item_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
