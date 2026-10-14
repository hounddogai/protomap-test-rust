use super::*;

use acme_kit::text;

/// How screens and exports show a primary slo target.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTargetView {
    pub code: String,
    pub label_text: String,
    pub grace_days: String,
    pub window_days: String,
    pub quota_units: String,
    pub discount: String,
    pub cipher: String,
    pub ttl_seconds: String,
    pub retries: String,
    pub notes: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_primary_slo_target_view(item: &PrimarySloTarget) -> PrimarySloTargetView {
    PrimarySloTargetView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        window_days: item.window_days.to_string(),
        quota_units: item.quota_units.to_string(),
        discount: format!("{:.2}", item.discount),
        cipher: text::truncate(&item.cipher, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        retries: item.retries.to_string(),
        notes: item.notes.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_slo_target_views(items: &[PrimarySloTarget]) -> Vec<PrimarySloTargetView> {
    let mut views: Vec<PrimarySloTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_slo_target_view(item));
    }
    views
}

pub fn sum_primary_slo_target_view(view: &PrimarySloTargetView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_window_days as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_slo_target("code-574", "label-549");
        let view = to_primary_slo_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_slo_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
