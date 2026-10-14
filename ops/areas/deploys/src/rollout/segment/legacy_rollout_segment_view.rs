use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegmentView {
    pub code: String,
    pub title: String,
    pub active_text: String,
    pub key_id: String,
    pub quota_units: String,
    pub country_text: String,
    pub fingerprint: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_segment_view(item: &LegacyRolloutSegment) -> LegacyRolloutSegmentView {
    LegacyRolloutSegmentView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        active_text: format!("{}", item.active),
        key_id: item.key_id.clone(),
        quota_units: item.quota_units.to_string(),
        country_text: item.country.clone(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_segment_views(items: &[LegacyRolloutSegment]) -> Vec<LegacyRolloutSegmentView> {
    let mut views: Vec<LegacyRolloutSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_segment_view(item));
    }
    views
}

pub fn sum_legacy_rollout_segment_view(view: &LegacyRolloutSegmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollout_segment("code-84", "title-29");
        let view = to_legacy_rollout_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollout_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
