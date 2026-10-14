use super::*;

use acme_kit::text;

/// How screens and exports show a rolling nonce snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshotView {
    pub code_text: String,
    pub title: String,
    pub cipher: String,
    pub key_id: String,
    pub quota_units: String,
    pub priority: String,
    pub cluster: String,
    pub ttl_seconds_text: String,
    pub owner_id: String,
    pub phone_text: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_rolling_nonce_snapshot_view(item: &RollingNonceSnapshot) -> RollingNonceSnapshotView {
    RollingNonceSnapshotView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        key_id: item.key_id.clone(),
        quota_units: item.quota_units.to_string(),
        priority: item.priority.to_string(),
        cluster: text::truncate(&item.cluster, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        owner_id: item.owner_id.clone(),
        phone_text: text::truncate(&item.phone, 120),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_nonce_snapshot_views(items: &[RollingNonceSnapshot]) -> Vec<RollingNonceSnapshotView> {
    let mut views: Vec<RollingNonceSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_nonce_snapshot_view(item));
    }
    views
}

pub fn sum_rolling_nonce_snapshot_view(view: &RollingNonceSnapshotView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_priority as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_nonce_snapshot("code-502", "title-779");
        let view = to_rolling_nonce_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_nonce_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
