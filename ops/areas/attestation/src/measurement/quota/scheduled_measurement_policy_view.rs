use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled measurement policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementPolicyView {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub quota_units: String,
    pub expires_at: String,
    pub tenant_id_text: String,
    pub segment: String,
    pub visible: String,
    pub locale: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_scheduled_measurement_policy_view(item: &ScheduledMeasurementPolicy) -> ScheduledMeasurementPolicyView {
    ScheduledMeasurementPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        checksum: text::truncate(&item.checksum, 120),
        quota_units: item.quota_units.to_string(),
        expires_at: item.expires_at.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        segment: item.segment.clone(),
        visible: format!("{}", item.visible),
        locale: item.locale.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_measurement_policy_views(
    items: &[ScheduledMeasurementPolicy],
) -> Vec<ScheduledMeasurementPolicyView> {
    let mut views: Vec<ScheduledMeasurementPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_measurement_policy_view(item));
    }
    views
}

pub fn sum_scheduled_measurement_policy_view(view: &ScheduledMeasurementPolicyView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_expires_at as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_measurement_policy("code-525", "name-542");
        let view = to_scheduled_measurement_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_measurement_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
