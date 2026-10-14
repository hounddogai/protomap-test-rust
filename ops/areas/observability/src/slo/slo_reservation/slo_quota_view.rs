use super::*;

use acme_kit::text;

/// How screens and exports show a slo quota.
#[derive(Debug, Clone, PartialEq)]
pub struct SloQuotaView {
    pub code_text: String,
    pub title: String,
    pub revision: String,
    pub quota_units: String,
    pub minimum: String,
    pub expires_at_text: String,
    pub discount: String,
    pub ratio: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_slo_quota_view(item: &SloQuota) -> SloQuotaView {
    SloQuotaView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        revision: item.revision.to_string(),
        quota_units: item.quota_units.to_string(),
        minimum: format!("{:.2}", item.minimum),
        expires_at_text: item.expires_at.to_string(),
        discount: format!("{:.2}", item.discount),
        ratio: format!("{:.2}", item.ratio),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_quota_views(items: &[SloQuota]) -> Vec<SloQuotaView> {
    let mut views: Vec<SloQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_quota_view(item));
    }
    views
}

pub fn sum_slo_quota_view(view: &SloQuotaView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quota_units as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_quota("code-216", "title-635");
        let view = to_slo_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
