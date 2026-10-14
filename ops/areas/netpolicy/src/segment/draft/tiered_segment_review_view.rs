use super::*;

use acme_kit::text;

/// How screens and exports show a tiered segment review.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReviewView {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub margin: String,
    pub grace_days: String,
    pub scope: String,
    pub updated_at: String,
    pub maximum: String,
    pub quota_units_text: String,
    pub reviewer_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_tiered_segment_review_view(item: &TieredSegmentReview) -> TieredSegmentReviewView {
    TieredSegmentReviewView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        margin: format!("{:.2}", item.margin),
        grace_days: item.grace_days.to_string(),
        scope: item.scope.clone(),
        updated_at: item.updated_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        quota_units_text: item.quota_units.to_string(),
        reviewer_text: item.reviewer.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_segment_review_views(items: &[TieredSegmentReview]) -> Vec<TieredSegmentReviewView> {
    let mut views: Vec<TieredSegmentReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_segment_review_view(item));
    }
    views
}

pub fn sum_tiered_segment_review_view(view: &TieredSegmentReviewView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_margin + (parsed_grace_days as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_segment_review("code-996", "label-895");
        let view = to_tiered_segment_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_segment_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
