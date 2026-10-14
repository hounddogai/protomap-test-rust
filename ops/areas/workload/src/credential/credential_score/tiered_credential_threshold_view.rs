use super::*;

use acme_kit::text;

/// How screens and exports show a tiered credential threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdView {
    pub code: String,
    pub label_text: String,
    pub threshold: String,
    pub timezone_text: String,
    pub scope: String,
    pub pinned: String,
    pub batch_size_text: String,
    pub phone: String,
    pub revision: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_tiered_credential_threshold_view(item: &TieredCredentialThreshold) -> TieredCredentialThresholdView {
    TieredCredentialThresholdView {
        code: text::truncate(&item.code, 40),
        label_text: text::truncate(&item.label, 80),
        threshold: format!("{:.2}", item.threshold),
        timezone_text: item.timezone.clone(),
        scope: text::truncate(&item.scope, 120),
        pinned: format!("{}", item.pinned),
        batch_size_text: item.batch_size.to_string(),
        phone: text::truncate(&item.phone, 64),
        revision: item.revision.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_credential_threshold_views(items: &[TieredCredentialThreshold]) -> Vec<TieredCredentialThresholdView> {
    let mut views: Vec<TieredCredentialThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_credential_threshold_view(item));
    }
    views
}

pub fn sum_tiered_credential_threshold_view(view: &TieredCredentialThresholdView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_threshold + (parsed_batch_size as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_credential_threshold("code-57", "label-99");
        let view = to_tiered_credential_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_credential_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
