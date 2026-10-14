use super::*;

use acme_kit::text;

/// How screens and exports show a policy review.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReviewView {
    pub code: String,
    pub title: String,
    pub created_at: String,
    pub quantity: String,
    pub segment: String,
    pub version: String,
    pub visible_text: String,
    pub tenant_id: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_policy_review_view(item: &PolicyReview) -> PolicyReviewView {
    PolicyReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        quantity: item.quantity.to_string(),
        segment: item.segment.clone(),
        version: item.version.to_string(),
        visible_text: format!("{}", item.visible),
        tenant_id: text::truncate(&item.tenant_id, 120),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_review_views(items: &[PolicyReview]) -> Vec<PolicyReviewView> {
    let mut views: Vec<PolicyReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_review_view(item));
    }
    views
}

pub fn sum_policy_review_view(view: &PolicyReviewView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quantity as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_review("code-163", "title-813");
        let view = to_policy_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
