use super::*;

use acme_kit::text;

/// How screens and exports show a identity record.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordView {
    pub code_text: String,
    pub title_text: String,
    pub cost: String,
    pub ttl_seconds: String,
    pub category: String,
    pub cipher: String,
    pub region: String,
    pub quota_units: String,
    pub checksum: String,
    pub port: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_identity_record_view(item: &IdentityRecord) -> IdentityRecordView {
    IdentityRecordView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        ttl_seconds: item.ttl_seconds.to_string(),
        category: text::truncate(&item.category, 80),
        cipher: item.cipher.clone(),
        region: item.region.clone(),
        quota_units: item.quota_units.to_string(),
        checksum: text::truncate(&item.checksum, 64),
        port: item.port.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_record_views(items: &[IdentityRecord]) -> Vec<IdentityRecordView> {
    let mut views: Vec<IdentityRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_record_view(item));
    }
    views
}

pub fn sum_identity_record_view(view: &IdentityRecordView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_cost + (parsed_ttl_seconds as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_record("code-557", "title-116");
        let view = to_identity_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
