use super::*;

use acme_kit::text;

/// How screens and exports show a alert override.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverrideView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub fingerprint: String,
    pub audience: String,
    pub floor_value: String,
    pub ends_at: String,
    pub rate_text: String,
    pub version_text: String,
    pub notes: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_alert_override_view(item: &AlertOverride) -> AlertOverrideView {
    AlertOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        fingerprint: item.fingerprint.clone(),
        audience: text::truncate(&item.audience, 40),
        floor_value: format!("{:.2}", item.floor_value),
        ends_at: item.ends_at.to_string(),
        rate_text: format!("{:.2}", item.rate),
        version_text: item.version.to_string(),
        notes: item.notes.clone(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_override_views(items: &[AlertOverride]) -> Vec<AlertOverrideView> {
    let mut views: Vec<AlertOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_override_view(item));
    }
    views
}

pub fn sum_alert_override_view(view: &AlertOverrideView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_revision as f64 + parsed_floor_value + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_override("code-740", "name-338");
        let view = to_alert_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
