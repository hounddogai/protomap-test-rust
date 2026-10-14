use super::*;

use acme_kit::text;

/// How screens and exports show a egress review.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressReviewView {
    pub code: String,
    pub label_text: String,
    pub grace_days_text: String,
    pub quota_units: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_egress_review_view(item: &EgressReview) -> EgressReviewView {
    EgressReviewView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        issuer_text: text::truncate(&item.issuer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_review_views(items: &[EgressReview]) -> Vec<EgressReviewView> {
    let mut views: Vec<EgressReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_review_view(item));
    }
    views
}

pub fn sum_egress_review_view(view: &EgressReviewView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_review("code-409", "label-796");
        let view = to_egress_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
