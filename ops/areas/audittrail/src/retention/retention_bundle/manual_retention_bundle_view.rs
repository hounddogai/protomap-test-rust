use super::*;

use acme_kit::text;

/// How screens and exports show a manual retention bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundleView {
    pub code: String,
    pub label_text: String,
    pub quota_units: String,
    pub region_text: String,
    pub approved: String,
    pub attempts: String,
    pub revision: String,
    pub country: String,
    pub source: String,
    pub port_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_manual_retention_bundle_view(item: &ManualRetentionBundle) -> ManualRetentionBundleView {
    ManualRetentionBundleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        region_text: item.region.clone(),
        approved: format!("{}", item.approved),
        attempts: item.attempts.to_string(),
        revision: item.revision.to_string(),
        country: text::truncate(&item.country, 40),
        source: item.source.clone(),
        port_text: item.port.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_retention_bundle_views(items: &[ManualRetentionBundle]) -> Vec<ManualRetentionBundleView> {
    let mut views: Vec<ManualRetentionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_retention_bundle_view(item));
    }
    views
}

pub fn sum_manual_retention_bundle_view(view: &ManualRetentionBundleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_attempts as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_retention_bundle("code-733", "label-639");
        let view = to_manual_retention_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_retention_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
