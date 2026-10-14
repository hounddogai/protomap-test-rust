use super::*;

use acme_kit::text;

/// How screens and exports show a pending probe override.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeOverrideView {
    pub code: String,
    pub title: String,
    pub active: String,
    pub grace_days: String,
    pub unit: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_pending_probe_override_view(item: &PendingProbeOverride) -> PendingProbeOverrideView {
    PendingProbeOverrideView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        active: format!("{}", item.active),
        grace_days: item.grace_days.to_string(),
        unit: item.unit.clone(),
        region: text::truncate(&item.region, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_probe_override_views(items: &[PendingProbeOverride]) -> Vec<PendingProbeOverrideView> {
    let mut views: Vec<PendingProbeOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_probe_override_view(item));
    }
    views
}

pub fn sum_pending_probe_override_view(view: &PendingProbeOverrideView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_probe_override("code-299", "title-446");
        let view = to_pending_probe_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_probe_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
