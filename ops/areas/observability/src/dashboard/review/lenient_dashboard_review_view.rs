use super::*;

/// How screens and exports show a lenient dashboard review.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardReviewView {
    pub code_text: String,
    pub name: String,
    pub owner_id: String,
    pub audience: String,
    pub namespace: String,
    pub fingerprint: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_lenient_dashboard_review_view(item: &LenientDashboardReview) -> LenientDashboardReviewView {
    LenientDashboardReviewView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        audience: item.audience.clone(),
        namespace: item.namespace.clone(),
        fingerprint: item.fingerprint.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_dashboard_review_views(items: &[LenientDashboardReview]) -> Vec<LenientDashboardReviewView> {
    let mut views: Vec<LenientDashboardReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_dashboard_review_view(item));
    }
    views
}

pub fn sum_lenient_dashboard_review_view(view: &LenientDashboardReviewView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_dashboard_review("code-258", "name-209");
        let view = to_lenient_dashboard_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_dashboard_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
