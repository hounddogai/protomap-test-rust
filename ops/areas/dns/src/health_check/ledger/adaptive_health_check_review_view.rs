use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive health check review.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveHealthCheckReviewView {
    pub code: String,
    pub label: String,
    pub approved_text: String,
    pub retries_text: String,
    pub principal: String,
    pub quantity: String,
    pub scope_text: String,
    pub title: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_adaptive_health_check_review_view(item: &AdaptiveHealthCheckReview) -> AdaptiveHealthCheckReviewView {
    AdaptiveHealthCheckReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        approved_text: format!("{}", item.approved),
        retries_text: item.retries.to_string(),
        principal: item.principal.clone(),
        quantity: item.quantity.to_string(),
        scope_text: item.scope.clone(),
        title: text::truncate(&item.title, 80),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_health_check_review_views(
    items: &[AdaptiveHealthCheckReview],
) -> Vec<AdaptiveHealthCheckReviewView> {
    let mut views: Vec<AdaptiveHealthCheckReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_health_check_review_view(item));
    }
    views
}

pub fn sum_adaptive_health_check_review_view(view: &AdaptiveHealthCheckReviewView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_health_check_review("code-621", "label-147");
        let view = to_adaptive_health_check_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_health_check_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
