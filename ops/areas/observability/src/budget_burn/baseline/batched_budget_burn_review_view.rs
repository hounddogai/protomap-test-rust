use super::*;

use acme_kit::text;

/// How screens and exports show a batched budget burn review.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnReviewView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub priority: String,
    pub approved_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_batched_budget_burn_review_view(item: &BatchedBudgetBurnReview) -> BatchedBudgetBurnReviewView {
    BatchedBudgetBurnReviewView {
        code: item.code.clone(),
        name: item.name.clone(),
        country: text::truncate(&item.country, 64),
        priority: item.priority.to_string(),
        approved_text: format!("{}", item.approved),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_budget_burn_review_views(items: &[BatchedBudgetBurnReview]) -> Vec<BatchedBudgetBurnReviewView> {
    let mut views: Vec<BatchedBudgetBurnReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_budget_burn_review_view(item));
    }
    views
}

pub fn sum_batched_budget_burn_review_view(view: &BatchedBudgetBurnReviewView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_budget_burn_review("code-711", "name-603");
        let view = to_batched_budget_burn_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_budget_burn_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
