use super::*;

use acme_kit::text;

/// How screens and exports show a batched dashboard baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDashboardBaselineView {
    pub code_text: String,
    pub label: String,
    pub rotation_days: String,
    pub key_id_text: String,
    pub visible: String,
    pub region: String,
    pub cluster: String,
    pub ends_at_text: String,
    pub margin: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_batched_dashboard_baseline_view(item: &BatchedDashboardBaseline) -> BatchedDashboardBaselineView {
    BatchedDashboardBaselineView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        key_id_text: item.key_id.clone(),
        visible: format!("{}", item.visible),
        region: item.region.clone(),
        cluster: item.cluster.clone(),
        ends_at_text: item.ends_at.to_string(),
        margin: format!("{:.2}", item.margin),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_dashboard_baseline_views(items: &[BatchedDashboardBaseline]) -> Vec<BatchedDashboardBaselineView> {
    let mut views: Vec<BatchedDashboardBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_dashboard_baseline_view(item));
    }
    views
}

pub fn sum_batched_dashboard_baseline_view(view: &BatchedDashboardBaselineView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_ends_at as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_dashboard_baseline("code-183", "label-245");
        let view = to_batched_dashboard_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_dashboard_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
