use super::*;

use acme_kit::text;

/// How screens and exports show a global allowlist bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundleView {
    pub code: String,
    pub name: String,
    pub minimum_text: String,
    pub grace_days: String,
    pub window_days_text: String,
    pub ttl_seconds: String,
    pub expires_at_text: String,
    pub owner_id: String,
    pub approved_text: String,
    pub retries: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_global_allowlist_bundle_view(item: &GlobalAllowlistBundle) -> GlobalAllowlistBundleView {
    GlobalAllowlistBundleView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        grace_days: item.grace_days.to_string(),
        window_days_text: item.window_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        expires_at_text: item.expires_at.to_string(),
        owner_id: item.owner_id.clone(),
        approved_text: format!("{}", item.approved),
        retries: item.retries.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_allowlist_bundle_views(items: &[GlobalAllowlistBundle]) -> Vec<GlobalAllowlistBundleView> {
    let mut views: Vec<GlobalAllowlistBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_allowlist_bundle_view(item));
    }
    views
}

pub fn sum_global_allowlist_bundle_view(view: &GlobalAllowlistBundleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_minimum + (parsed_grace_days as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_allowlist_bundle("code-95", "name-42");
        let view = to_global_allowlist_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_allowlist_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
