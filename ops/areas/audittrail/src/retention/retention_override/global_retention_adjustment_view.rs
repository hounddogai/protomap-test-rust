use super::*;

use acme_kit::text;

/// How screens and exports show a global retention adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentView {
    pub code: String,
    pub label: String,
    pub threshold: String,
    pub owner_id_text: String,
    pub active: String,
    pub timezone: String,
    pub archived_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_global_retention_adjustment_view(item: &GlobalRetentionAdjustment) -> GlobalRetentionAdjustmentView {
    GlobalRetentionAdjustmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        owner_id_text: text::truncate(&item.owner_id, 120),
        active: format!("{}", item.active),
        timezone: item.timezone.clone(),
        archived_text: format!("{}", item.archived),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_retention_adjustment_views(items: &[GlobalRetentionAdjustment]) -> Vec<GlobalRetentionAdjustmentView> {
    let mut views: Vec<GlobalRetentionAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_retention_adjustment_view(item));
    }
    views
}

pub fn sum_global_retention_adjustment_view(view: &GlobalRetentionAdjustmentView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_retention_adjustment("code-938", "label-504");
        let view = to_global_retention_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_retention_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
