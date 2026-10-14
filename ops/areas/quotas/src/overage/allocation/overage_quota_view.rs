use super::*;

use acme_kit::text;

/// How screens and exports show a overage quota.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageQuotaView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub unit_text: String,
    pub starts_at: String,
    pub external_ref: String,
    pub notes_text: String,
    pub key_id_text: String,
    pub attempts: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_overage_quota_view(item: &OverageQuota) -> OverageQuotaView {
    OverageQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        updated_at: item.updated_at.to_string(),
        unit_text: item.unit.clone(),
        starts_at: item.starts_at.to_string(),
        external_ref: item.external_ref.clone(),
        notes_text: item.notes.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        attempts: item.attempts.to_string(),
        phone: text::truncate(&item.phone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_quota_views(items: &[OverageQuota]) -> Vec<OverageQuotaView> {
    let mut views: Vec<OverageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_quota_view(item));
    }
    views
}

pub fn sum_overage_quota_view(view: &OverageQuotaView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_starts_at as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_quota("code-180", "name-667");
        let view = to_overage_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
