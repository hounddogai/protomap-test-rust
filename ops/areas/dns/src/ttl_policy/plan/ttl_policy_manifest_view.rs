use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyManifestView {
    pub code_text: String,
    pub title: String,
    pub fingerprint: String,
    pub locale_text: String,
    pub reviewer: String,
    pub weight: String,
    pub scope: String,
    pub summary: String,
    pub margin_text: String,
    pub duration_minutes_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_ttl_policy_manifest_view(item: &TtlPolicyManifest) -> TtlPolicyManifestView {
    TtlPolicyManifestView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        fingerprint: item.fingerprint.clone(),
        locale_text: item.locale.clone(),
        reviewer: item.reviewer.clone(),
        weight: format!("{:.2}", item.weight),
        scope: text::truncate(&item.scope, 40),
        summary: item.summary.clone(),
        margin_text: format!("{:.2}", item.margin),
        duration_minutes_text: item.duration_minutes.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_manifest_views(items: &[TtlPolicyManifest]) -> Vec<TtlPolicyManifestView> {
    let mut views: Vec<TtlPolicyManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_manifest_view(item));
    }
    views
}

pub fn sum_ttl_policy_manifest_view(view: &TtlPolicyManifestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_weight + parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_manifest("code-94", "title-88");
        let view = to_ttl_policy_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
