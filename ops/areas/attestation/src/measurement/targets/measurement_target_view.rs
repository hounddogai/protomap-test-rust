use super::*;

use acme_kit::text;

/// How screens and exports show a measurement target.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTargetView {
    pub code_text: String,
    pub name: String,
    pub revision: String,
    pub active_text: String,
    pub currency: String,
    pub version: String,
    pub key_id: String,
    pub channel: String,
    pub quota_units: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_measurement_target_view(item: &MeasurementTarget) -> MeasurementTargetView {
    MeasurementTargetView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        active_text: format!("{}", item.active),
        currency: item.currency.clone(),
        version: item.version.to_string(),
        key_id: item.key_id.clone(),
        channel: text::truncate(&item.channel, 64),
        quota_units: item.quota_units.to_string(),
        label: text::truncate(&item.label, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_target_views(items: &[MeasurementTarget]) -> Vec<MeasurementTargetView> {
    let mut views: Vec<MeasurementTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_target_view(item));
    }
    views
}

pub fn sum_measurement_target_view(view: &MeasurementTargetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_version as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_target("code-175", "name-970");
        let view = to_measurement_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
