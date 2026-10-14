use super::*;

use acme_kit::text;

/// How screens and exports show a rolling alert manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertManifestView {
    pub code: String,
    pub label_text: String,
    pub margin: String,
    pub visible: String,
    pub checksum: String,
    pub created_at_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_rolling_alert_manifest_view(item: &RollingAlertManifest) -> RollingAlertManifestView {
    RollingAlertManifestView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        margin: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        checksum: item.checksum.clone(),
        created_at_text: item.created_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_alert_manifest_views(items: &[RollingAlertManifest]) -> Vec<RollingAlertManifestView> {
    let mut views: Vec<RollingAlertManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_alert_manifest_view(item));
    }
    views
}

pub fn sum_rolling_alert_manifest_view(view: &RollingAlertManifestView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_margin + (parsed_created_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_alert_manifest("code-117", "label-241");
        let view = to_rolling_alert_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_alert_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
