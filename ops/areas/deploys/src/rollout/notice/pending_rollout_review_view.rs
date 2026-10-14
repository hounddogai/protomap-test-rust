use super::*;

use acme_kit::text;

/// How screens and exports show a pending rollout review.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReviewView {
    pub code_text: String,
    pub label: String,
    pub title_text: String,
    pub budget: String,
    pub segment: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_pending_rollout_review_view(item: &PendingRolloutReview) -> PendingRolloutReviewView {
    PendingRolloutReviewView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        title_text: text::truncate(&item.title, 120),
        budget: format!("{:.2}", item.budget),
        segment: item.segment.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_rollout_review_views(items: &[PendingRolloutReview]) -> Vec<PendingRolloutReviewView> {
    let mut views: Vec<PendingRolloutReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_rollout_review_view(item));
    }
    views
}

pub fn sum_pending_rollout_review_view(view: &PendingRolloutReviewView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_budget + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_rollout_review("code-501", "label-753");
        let view = to_pending_rollout_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_rollout_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
