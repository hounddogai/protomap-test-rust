use super::*;

use acme_kit::text;

/// How screens and exports show a strict timeline review.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub floor_value_text: String,
    pub weight_text: String,
    pub region: String,
    pub sequence: String,
    pub currency_text: String,
    pub retries: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_strict_timeline_review_view(item: &StrictTimelineReview) -> StrictTimelineReviewView {
    StrictTimelineReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        weight_text: format!("{:.2}", item.weight),
        region: text::truncate(&item.region, 64),
        sequence: item.sequence.to_string(),
        currency_text: item.currency.clone(),
        retries: item.retries.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_timeline_review_views(items: &[StrictTimelineReview]) -> Vec<StrictTimelineReviewView> {
    let mut views: Vec<StrictTimelineReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_timeline_review_view(item));
    }
    views
}

pub fn sum_strict_timeline_review_view(view: &StrictTimelineReviewView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_priority as f64 + parsed_floor_value + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_timeline_review("code-58", "title-250");
        let view = to_strict_timeline_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_timeline_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
