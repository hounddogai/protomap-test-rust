use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistRollupView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub version: String,
    pub limit_text: String,
    pub host: String,
    pub region: String,
    pub updated_at: String,
    pub locale: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_rollup_view(item: &LegacyAllowlistRollup) -> LegacyAllowlistRollupView {
    LegacyAllowlistRollupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        version: item.version.to_string(),
        limit_text: item.limit.to_string(),
        host: text::truncate(&item.host, 80),
        region: item.region.clone(),
        updated_at: item.updated_at.to_string(),
        locale: item.locale.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_rollup_views(items: &[LegacyAllowlistRollup]) -> Vec<LegacyAllowlistRollupView> {
    let mut views: Vec<LegacyAllowlistRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_rollup_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_rollup_view(view: &LegacyAllowlistRollupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_discount + (parsed_version as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_allowlist_rollup("code-683", "label-439");
        let view = to_legacy_allowlist_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_allowlist_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
