use super::*;

use acme_kit::text;

/// How screens and exports show a grant review.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantReviewView {
    pub code_text: String,
    pub label_text: String,
    pub budget: String,
    pub issuer: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_grant_review_view(item: &GrantReview) -> GrantReviewView {
    GrantReviewView {
        code_text: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        budget: format!("{:.2}", item.budget),
        issuer: item.issuer.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_review_views(items: &[GrantReview]) -> Vec<GrantReviewView> {
    let mut views: Vec<GrantReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_review_view(item));
    }
    views
}

pub fn sum_grant_review_view(view: &GrantReviewView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_review("code-715", "label-856");
        let view = to_grant_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
