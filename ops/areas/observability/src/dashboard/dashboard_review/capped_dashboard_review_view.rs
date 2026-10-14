use super::*;

use acme_kit::text;

/// How screens and exports show a capped dashboard review.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardReviewView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub locale: String,
    pub limit: String,
    pub duration_minutes: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_capped_dashboard_review_view(item: &CappedDashboardReview) -> CappedDashboardReviewView {
    CappedDashboardReviewView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        locale: item.locale.clone(),
        limit: item.limit.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_dashboard_review_views(items: &[CappedDashboardReview]) -> Vec<CappedDashboardReviewView> {
    let mut views: Vec<CappedDashboardReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_dashboard_review_view(item));
    }
    views
}

pub fn sum_capped_dashboard_review_view(view: &CappedDashboardReviewView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_limit as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_dashboard_review("code-773", "label-639");
        let view = to_capped_dashboard_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_dashboard_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
