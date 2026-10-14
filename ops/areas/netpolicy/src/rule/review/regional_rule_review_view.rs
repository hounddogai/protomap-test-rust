use super::*;

use acme_kit::text;

/// How screens and exports show a regional rule review.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleReviewView {
    pub code: String,
    pub label_text: String,
    pub reviewer: String,
    pub contact_email: String,
    pub quantity: String,
    pub name_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_regional_rule_review_view(item: &RegionalRuleReview) -> RegionalRuleReviewView {
    RegionalRuleReviewView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        reviewer: item.reviewer.clone(),
        contact_email: item.contact_email.clone(),
        quantity: item.quantity.to_string(),
        name_text: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_rule_review_views(items: &[RegionalRuleReview]) -> Vec<RegionalRuleReviewView> {
    let mut views: Vec<RegionalRuleReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_rule_review_view(item));
    }
    views
}

pub fn sum_regional_rule_review_view(view: &RegionalRuleReviewView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_rule_review("code-446", "label-871");
        let view = to_regional_rule_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_rule_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
