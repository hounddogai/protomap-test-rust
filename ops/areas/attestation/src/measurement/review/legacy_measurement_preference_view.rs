use super::*;

use acme_kit::text;

/// How screens and exports show a legacy measurement preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreferenceView {
    pub code: String,
    pub name: String,
    pub maximum: String,
    pub capacity_text: String,
    pub contact_email: String,
    pub pinned: String,
    pub ttl_seconds: String,
    pub limit: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_legacy_measurement_preference_view(item: &LegacyMeasurementPreference) -> LegacyMeasurementPreferenceView {
    LegacyMeasurementPreferenceView {
        code: item.code.clone(),
        name: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        capacity_text: item.capacity.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        pinned: format!("{}", item.pinned),
        ttl_seconds: item.ttl_seconds.to_string(),
        limit: item.limit.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_measurement_preference_views(
    items: &[LegacyMeasurementPreference],
) -> Vec<LegacyMeasurementPreferenceView> {
    let mut views: Vec<LegacyMeasurementPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_measurement_preference_view(item));
    }
    views
}

pub fn sum_legacy_measurement_preference_view(view: &LegacyMeasurementPreferenceView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_maximum + (parsed_capacity as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_measurement_preference("code-786", "name-891");
        let view = to_legacy_measurement_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_measurement_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
