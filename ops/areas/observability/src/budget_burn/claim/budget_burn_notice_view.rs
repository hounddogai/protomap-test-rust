use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn notice.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnNoticeView {
    pub code_text: String,
    pub label_text: String,
    pub issuer: String,
    pub namespace: String,
    pub pinned: String,
    pub priority: String,
    pub sequence: String,
    pub description_text: String,
    pub created_at: String,
    pub quota_units: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_budget_burn_notice_view(item: &BudgetBurnNotice) -> BudgetBurnNoticeView {
    BudgetBurnNoticeView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        issuer: item.issuer.clone(),
        namespace: text::truncate(&item.namespace, 64),
        pinned: format!("{}", item.pinned),
        priority: item.priority.to_string(),
        sequence: item.sequence.to_string(),
        description_text: item.description.clone(),
        created_at: item.created_at.to_string(),
        quota_units: item.quota_units.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_notice_views(items: &[BudgetBurnNotice]) -> Vec<BudgetBurnNoticeView> {
    let mut views: Vec<BudgetBurnNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_notice_view(item));
    }
    views
}

pub fn sum_budget_burn_notice_view(view: &BudgetBurnNoticeView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_sequence as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_notice("code-219", "label-810");
        let view = to_budget_burn_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
