use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuotaView {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub visible: String,
    pub phone: String,
    pub minimum: String,
    pub grace_days: String,
    pub checksum: String,
    pub notes_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_quota_view(item: &LegacyAllowlistQuota) -> LegacyAllowlistQuotaView {
    LegacyAllowlistQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 120),
        visible: format!("{}", item.visible),
        phone: text::truncate(&item.phone, 120),
        minimum: format!("{:.2}", item.minimum),
        grace_days: item.grace_days.to_string(),
        checksum: item.checksum.clone(),
        notes_text: item.notes.clone(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_quota_views(items: &[LegacyAllowlistQuota]) -> Vec<LegacyAllowlistQuotaView> {
    let mut views: Vec<LegacyAllowlistQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_quota_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_quota_view(view: &LegacyAllowlistQuotaView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_minimum + (parsed_grace_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_allowlist_quota("code-71", "title-580");
        let view = to_legacy_allowlist_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_allowlist_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
