use super::*;

use acme_kit::text;

/// How screens and exports show a strict approval adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalAdjustmentView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub amount: String,
    pub grace_days: String,
    pub visible: String,
    pub created_at_text: String,
    pub country_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_strict_approval_adjustment_view(item: &StrictApprovalAdjustment) -> StrictApprovalAdjustmentView {
    StrictApprovalAdjustmentView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        namespace: item.namespace.clone(),
        amount: format!("{:.2}", item.amount),
        grace_days: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        created_at_text: item.created_at.to_string(),
        country_text: item.country.clone(),
        audience: text::truncate(&item.audience, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_approval_adjustment_views(items: &[StrictApprovalAdjustment]) -> Vec<StrictApprovalAdjustmentView> {
    let mut views: Vec<StrictApprovalAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_approval_adjustment_view(item));
    }
    views
}

pub fn sum_strict_approval_adjustment_view(view: &StrictApprovalAdjustmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_amount + (parsed_grace_days as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_approval_adjustment("code-725", "name-535");
        let view = to_strict_approval_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_approval_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
