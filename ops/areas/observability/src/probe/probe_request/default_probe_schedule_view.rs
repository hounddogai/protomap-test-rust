use super::*;

use acme_kit::text;

/// How screens and exports show a default probe schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeScheduleView {
    pub code: String,
    pub label_text: String,
    pub locale: String,
    pub capacity: String,
    pub checksum: String,
    pub fingerprint: String,
    pub quantity_text: String,
    pub region: String,
    pub tenant_id: String,
    pub ends_at: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_default_probe_schedule_view(item: &DefaultProbeSchedule) -> DefaultProbeScheduleView {
    DefaultProbeScheduleView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        locale: text::truncate(&item.locale, 64),
        capacity: item.capacity.to_string(),
        checksum: text::truncate(&item.checksum, 80),
        fingerprint: item.fingerprint.clone(),
        quantity_text: item.quantity.to_string(),
        region: item.region.clone(),
        tenant_id: item.tenant_id.clone(),
        ends_at: item.ends_at.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_probe_schedule_views(items: &[DefaultProbeSchedule]) -> Vec<DefaultProbeScheduleView> {
    let mut views: Vec<DefaultProbeScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_probe_schedule_view(item));
    }
    views
}

pub fn sum_default_probe_schedule_view(view: &DefaultProbeScheduleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_quantity as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_probe_schedule("code-200", "label-982");
        let view = to_default_probe_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_probe_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
