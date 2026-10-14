use super::*;

use acme_kit::text;

/// How screens and exports show a strict failover review.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReviewView {
    pub code: String,
    pub title_text: String,
    pub key_id: String,
    pub namespace: String,
    pub priority_text: String,
    pub retries_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_strict_failover_review_view(item: &StrictFailoverReview) -> StrictFailoverReviewView {
    StrictFailoverReviewView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        key_id: text::truncate(&item.key_id, 40),
        namespace: text::truncate(&item.namespace, 40),
        priority_text: item.priority.to_string(),
        retries_text: item.retries.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_failover_review_views(items: &[StrictFailoverReview]) -> Vec<StrictFailoverReviewView> {
    let mut views: Vec<StrictFailoverReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_failover_review_view(item));
    }
    views
}

pub fn sum_strict_failover_review_view(view: &StrictFailoverReviewView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_retries as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_failover_review("code-357", "title-756");
        let view = to_strict_failover_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_failover_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
