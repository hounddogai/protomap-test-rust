use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout window.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindowView {
    pub code: String,
    pub name: String,
    pub starts_at: String,
    pub cluster: String,
    pub category_text: String,
    pub currency: String,
    pub source: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_window_view(item: &LegacyRolloutWindow) -> LegacyRolloutWindowView {
    LegacyRolloutWindowView {
        code: item.code.clone(),
        name: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        cluster: item.cluster.clone(),
        category_text: text::truncate(&item.category, 120),
        currency: item.currency.clone(),
        source: item.source.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_window_views(items: &[LegacyRolloutWindow]) -> Vec<LegacyRolloutWindowView> {
    let mut views: Vec<LegacyRolloutWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_window_view(item));
    }
    views
}

pub fn sum_legacy_rollout_window_view(view: &LegacyRolloutWindowView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollout_window("code-828", "name-869");
        let view = to_legacy_rollout_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollout_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
