use super::*;

use acme_kit::text;

/// How screens and exports show a overage review.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewView {
    pub code_text: String,
    pub name_text: String,
    pub owner_id: String,
    pub ends_at: String,
    pub amount_text: String,
    pub external_ref: String,
    pub visible: String,
    pub created_at: String,
    pub cluster: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_overage_review_view(item: &OverageReview) -> OverageReviewView {
    OverageReviewView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        owner_id: item.owner_id.clone(),
        ends_at: item.ends_at.to_string(),
        amount_text: format!("{:.2}", item.amount),
        external_ref: text::truncate(&item.external_ref, 64),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        cluster: text::truncate(&item.cluster, 120),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_review_views(items: &[OverageReview]) -> Vec<OverageReviewView> {
    let mut views: Vec<OverageReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_review_view(item));
    }
    views
}

pub fn sum_overage_review_view(view: &OverageReviewView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_amount + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_review("code-481", "name-507");
        let view = to_overage_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
