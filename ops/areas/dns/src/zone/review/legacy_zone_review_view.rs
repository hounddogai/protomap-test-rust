use super::*;

use acme_kit::text;

/// How screens and exports show a legacy zone review.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub quota_units: String,
    pub key_id: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_legacy_zone_review_view(item: &LegacyZoneReview) -> LegacyZoneReviewView {
    LegacyZoneReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        quota_units: item.quota_units.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_zone_review_views(items: &[LegacyZoneReview]) -> Vec<LegacyZoneReviewView> {
    let mut views: Vec<LegacyZoneReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_zone_review_view(item));
    }
    views
}

pub fn sum_legacy_zone_review_view(view: &LegacyZoneReviewView) -> Result<f64, String> {
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
        let item = new_legacy_zone_review("code-517", "title-259");
        let view = to_legacy_zone_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_zone_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
