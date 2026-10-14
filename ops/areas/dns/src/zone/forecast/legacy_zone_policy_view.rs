use super::*;

use acme_kit::text;

/// How screens and exports show a legacy zone policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub currency_text: String,
    pub quota_units: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_legacy_zone_policy_view(item: &LegacyZonePolicy) -> LegacyZonePolicyView {
    LegacyZonePolicyView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        description: item.description.clone(),
        currency_text: item.currency.clone(),
        quota_units: item.quota_units.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_zone_policy_views(items: &[LegacyZonePolicy]) -> Vec<LegacyZonePolicyView> {
    let mut views: Vec<LegacyZonePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_zone_policy_view(item));
    }
    views
}

pub fn sum_legacy_zone_policy_view(view: &LegacyZonePolicyView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_zone_policy("code-743", "title-94");
        let view = to_legacy_zone_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_zone_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
