use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundleView {
    pub code: String,
    pub name: String,
    pub cluster_text: String,
    pub sequence_text: String,
    pub locale_text: String,
    pub capacity_text: String,
    pub attempts: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_bundle_view(item: &LegacyAllowlistBundle) -> LegacyAllowlistBundleView {
    LegacyAllowlistBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        cluster_text: item.cluster.clone(),
        sequence_text: item.sequence.to_string(),
        locale_text: text::truncate(&item.locale, 64),
        capacity_text: item.capacity.to_string(),
        attempts: item.attempts.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_bundle_views(items: &[LegacyAllowlistBundle]) -> Vec<LegacyAllowlistBundleView> {
    let mut views: Vec<LegacyAllowlistBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_bundle_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_bundle_view(view: &LegacyAllowlistBundleView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_capacity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_allowlist_bundle("code-741", "name-92");
        let view = to_legacy_allowlist_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_allowlist_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
