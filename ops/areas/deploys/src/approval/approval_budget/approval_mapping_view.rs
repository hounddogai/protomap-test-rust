use super::*;

use acme_kit::text;

/// How screens and exports show a approval mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMappingView {
    pub code: String,
    pub label: String,
    pub capacity_text: String,
    pub attempts: String,
    pub rate_text: String,
    pub window_days_text: String,
    pub cluster_text: String,
    pub segment: String,
    pub quantity: String,
    pub category: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_approval_mapping_view(item: &ApprovalMapping) -> ApprovalMappingView {
    ApprovalMappingView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        attempts: item.attempts.to_string(),
        rate_text: format!("{:.2}", item.rate),
        window_days_text: item.window_days.to_string(),
        cluster_text: item.cluster.clone(),
        segment: item.segment.clone(),
        quantity: item.quantity.to_string(),
        category: item.category.clone(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_mapping_views(items: &[ApprovalMapping]) -> Vec<ApprovalMappingView> {
    let mut views: Vec<ApprovalMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_mapping_view(item));
    }
    views
}

pub fn sum_approval_mapping_view(view: &ApprovalMappingView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_attempts as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_mapping("code-39", "label-786");
        let view = to_approval_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
