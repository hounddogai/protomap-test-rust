use super::*;

/// How screens and exports show a pending probe adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAdjustmentView {
    pub code: String,
    pub label_text: String,
    pub quota_units: String,
    pub capacity: String,
    pub pinned: String,
    pub unit_text: String,
    pub starts_at_text: String,
    pub maximum: String,
    pub archived: String,
    pub locale: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_pending_probe_adjustment_view(item: &PendingProbeAdjustment) -> PendingProbeAdjustmentView {
    PendingProbeAdjustmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        capacity: item.capacity.to_string(),
        pinned: format!("{}", item.pinned),
        unit_text: item.unit.clone(),
        starts_at_text: item.starts_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        archived: format!("{}", item.archived),
        locale: item.locale.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_probe_adjustment_views(items: &[PendingProbeAdjustment]) -> Vec<PendingProbeAdjustmentView> {
    let mut views: Vec<PendingProbeAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_probe_adjustment_view(item));
    }
    views
}

pub fn sum_pending_probe_adjustment_view(view: &PendingProbeAdjustmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_capacity as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_probe_adjustment("code-69", "label-241");
        let view = to_pending_probe_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_probe_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
