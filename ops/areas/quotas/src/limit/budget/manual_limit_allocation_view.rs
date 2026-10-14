use super::*;

/// How screens and exports show a manual limit allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitAllocationView {
    pub code: String,
    pub title: String,
    pub checksum_text: String,
    pub amount: String,
    pub unit: String,
    pub visible_text: String,
    pub attempts: String,
    pub revision: String,
    pub cost_text: String,
    pub maximum: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_manual_limit_allocation_view(item: &ManualLimitAllocation) -> ManualLimitAllocationView {
    ManualLimitAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        checksum_text: item.checksum.clone(),
        amount: format!("{:.2}", item.amount),
        unit: item.unit.clone(),
        visible_text: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        revision: item.revision.to_string(),
        cost_text: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_limit_allocation_views(items: &[ManualLimitAllocation]) -> Vec<ManualLimitAllocationView> {
    let mut views: Vec<ManualLimitAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_limit_allocation_view(item));
    }
    views
}

pub fn sum_manual_limit_allocation_view(view: &ManualLimitAllocationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_amount + (parsed_attempts as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_limit_allocation("code-636", "title-545");
        let view = to_manual_limit_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_limit_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
