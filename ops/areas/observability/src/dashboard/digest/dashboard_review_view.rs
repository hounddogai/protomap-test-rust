use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard review.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReviewView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub starts_at: String,
    pub visible: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_dashboard_review_view(item: &DashboardReview) -> DashboardReviewView {
    DashboardReviewView {
        code: item.code.clone(),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        starts_at: item.starts_at.to_string(),
        visible: format!("{}", item.visible),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_review_views(items: &[DashboardReview]) -> Vec<DashboardReviewView> {
    let mut views: Vec<DashboardReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_review_view(item));
    }
    views
}

pub fn sum_dashboard_review_view(view: &DashboardReviewView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_review("code-162", "name-856");
        let view = to_dashboard_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
