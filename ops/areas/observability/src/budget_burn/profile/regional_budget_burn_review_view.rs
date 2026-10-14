use super::*;

/// How screens and exports show a regional budget burn review.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReviewView {
    pub code: String,
    pub name: String,
    pub floor_value: String,
    pub rotation_days: String,
    pub cost: String,
    pub capacity: String,
    pub scope_text: String,
    pub minimum: String,
    pub threshold: String,
    pub grace_days_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_regional_budget_burn_review_view(item: &RegionalBudgetBurnReview) -> RegionalBudgetBurnReviewView {
    RegionalBudgetBurnReviewView {
        code: item.code.clone(),
        name: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        rotation_days: item.rotation_days.to_string(),
        cost: format!("{:.2}", item.cost),
        capacity: item.capacity.to_string(),
        scope_text: item.scope.clone(),
        minimum: format!("{:.2}", item.minimum),
        threshold: format!("{:.2}", item.threshold),
        grace_days_text: item.grace_days.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_budget_burn_review_views(items: &[RegionalBudgetBurnReview]) -> Vec<RegionalBudgetBurnReviewView> {
    let mut views: Vec<RegionalBudgetBurnReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_budget_burn_review_view(item));
    }
    views
}

pub fn sum_regional_budget_burn_review_view(view: &RegionalBudgetBurnReviewView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_floor_value + (parsed_rotation_days as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_budget_burn_review("code-546", "name-402");
        let view = to_regional_budget_burn_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_budget_burn_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
