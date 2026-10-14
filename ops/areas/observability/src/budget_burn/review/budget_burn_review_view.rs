use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn review.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewView {
    pub code: String,
    pub title_text: String,
    pub ratio: String,
    pub budget_text: String,
    pub principal: String,
    pub maximum_text: String,
    pub fingerprint: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_budget_burn_review_view(item: &BudgetBurnReview) -> BudgetBurnReviewView {
    BudgetBurnReviewView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        budget_text: format!("{:.2}", item.budget),
        principal: item.principal.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        fingerprint: item.fingerprint.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_review_views(items: &[BudgetBurnReview]) -> Vec<BudgetBurnReviewView> {
    let mut views: Vec<BudgetBurnReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_review_view(item));
    }
    views
}

pub fn sum_budget_burn_review_view(view: &BudgetBurnReviewView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ratio + parsed_budget + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_review("code-454", "title-984");
        let view = to_budget_burn_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
