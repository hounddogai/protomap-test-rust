use super::*;

use acme_kit::text;

/// How screens and exports show a primary identity review.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityReviewView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub channel: String,
    pub key_id: String,
    pub notes: String,
    pub audience: String,
    pub priority: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_primary_identity_review_view(item: &PrimaryIdentityReview) -> PrimaryIdentityReviewView {
    PrimaryIdentityReviewView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        channel: text::truncate(&item.channel, 80),
        key_id: item.key_id.clone(),
        notes: item.notes.clone(),
        audience: item.audience.clone(),
        priority: item.priority.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_identity_review_views(items: &[PrimaryIdentityReview]) -> Vec<PrimaryIdentityReviewView> {
    let mut views: Vec<PrimaryIdentityReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_identity_review_view(item));
    }
    views
}

pub fn sum_primary_identity_review_view(view: &PrimaryIdentityReviewView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_cost + (parsed_priority as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_identity_review("code-792", "label-314");
        let view = to_primary_identity_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_identity_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
