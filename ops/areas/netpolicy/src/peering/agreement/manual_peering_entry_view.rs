use super::*;

use acme_kit::text;

/// How screens and exports show a manual peering entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringEntryView {
    pub code: String,
    pub label_text: String,
    pub weight: String,
    pub fingerprint: String,
    pub visible: String,
    pub created_at: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_manual_peering_entry_view(item: &ManualPeeringEntry) -> ManualPeeringEntryView {
    ManualPeeringEntryView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 120),
        weight: format!("{:.2}", item.weight),
        fingerprint: text::truncate(&item.fingerprint, 80),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_peering_entry_views(items: &[ManualPeeringEntry]) -> Vec<ManualPeeringEntryView> {
    let mut views: Vec<ManualPeeringEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_peering_entry_view(item));
    }
    views
}

pub fn sum_manual_peering_entry_view(view: &ManualPeeringEntryView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_weight + (parsed_created_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_peering_entry("code-192", "label-805");
        let view = to_manual_peering_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_peering_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
