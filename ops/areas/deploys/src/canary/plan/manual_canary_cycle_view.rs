use super::*;

use acme_kit::text;

/// How screens and exports show a manual canary cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanaryCycleView {
    pub code_text: String,
    pub name: String,
    pub checksum: String,
    pub pinned_text: String,
    pub grace_days_text: String,
    pub quota_units: String,
    pub burst: String,
    pub owner_id_text: String,
    pub rotation_days: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_manual_canary_cycle_view(item: &ManualCanaryCycle) -> ManualCanaryCycleView {
    ManualCanaryCycleView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        checksum: text::truncate(&item.checksum, 64),
        pinned_text: format!("{}", item.pinned),
        grace_days_text: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        burst: item.burst.to_string(),
        owner_id_text: item.owner_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_canary_cycle_views(items: &[ManualCanaryCycle]) -> Vec<ManualCanaryCycleView> {
    let mut views: Vec<ManualCanaryCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_canary_cycle_view(item));
    }
    views
}

pub fn sum_manual_canary_cycle_view(view: &ManualCanaryCycleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_quota_units as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_canary_cycle("code-68", "name-391");
        let view = to_manual_canary_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_canary_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
